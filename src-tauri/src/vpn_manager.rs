use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use log::{error, info, warn};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use crate::config_gen;
use crate::geo_db::GeoDatabase;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct VpnStatus {
    pub connected: bool,
    pub current_node_id: Option<String>,
    pub current_node_name: Option<String>,
    pub connected_at: Option<i64>,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub total_download: u64,
    pub total_upload: u64,
    pub latency_ms: Option<f64>,
}

pub struct VpnManager {
    app: AppHandle,
    process: Option<Child>,
    status: VpnStatus,
    config_dir: PathBuf,
    sing_box_path: PathBuf,
    pub db: Option<Arc<Mutex<rusqlite::Connection>>>,
    #[allow(dead_code)]
    speed_history: Vec<SpeedSample>,
    logs: Vec<LogEntry>,
    auto_switch_task: Option<tokio::task::JoinHandle<()>>,
    subscription_refresh_task: Option<tokio::task::JoinHandle<()>>,
    auto_ping_task: Option<tokio::task::JoinHandle<()>>,
    /// Weak reference to self Arc, injected after construction in lib.rs
    pub self_ref: Option<std::sync::Weak<tokio::sync::Mutex<VpnManager>>>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct SpeedSample {
    timestamp: i64,
    download: f64,
    upload: f64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

impl VpnManager {
    pub fn new(app: AppHandle) -> Self {
        let resource_dir = app
            .path()
            .resource_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

        // Use app_data_dir for runtime files (config, logs) — outside project tree
        let app_data = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
        let config_dir = app_data.join("config");
        std::fs::create_dir_all(&config_dir).ok();

        // Binary path: check resource_dir first, then manifest_dir (dev mode fallback)
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let bin_name = if cfg!(target_os = "windows") { "sing-box.exe" } else { "sing-box" };
        let sing_box_path = if resource_dir.join("bin").join(bin_name).exists() {
            resource_dir.join("bin").join(bin_name)
        } else if manifest_dir.join("bin").join(bin_name).exists() {
            manifest_dir.join("bin").join(bin_name)
        } else {
            resource_dir.join("bin").join(bin_name)
        };

        Self {
            app,
            process: None,
            status: VpnStatus {
                connected: false,
                current_node_id: None,
                current_node_name: None,
                connected_at: None,
                download_speed: 0.0,
                upload_speed: 0.0,
                total_download: 0,
                total_upload: 0,
                latency_ms: None,
            },
            config_dir,
            sing_box_path,
            db: None,
            speed_history: Vec::new(),
            logs: Vec::new(),
            auto_switch_task: None,
            subscription_refresh_task: None,
            auto_ping_task: None,
            self_ref: None,
        }
    }

    pub fn init_db(&mut self) {
        let db_path = self.app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from(".")).join("client.db");
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        match rusqlite::Connection::open(&db_path) {
            Ok(conn) => {
                conn.execute_batch(
                    "
                    CREATE TABLE IF NOT EXISTS nodes (
                        id TEXT PRIMARY KEY,
                        name TEXT NOT NULL,
                        protocol TEXT NOT NULL,
                        server TEXT NOT NULL,
                        port INTEGER NOT NULL,
                        config TEXT NOT NULL,
                        latency_ms REAL,
                        uptime REAL,
                        error_count INTEGER DEFAULT 0,
                        last_test_at TEXT,
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL,
                        enabled INTEGER DEFAULT 1,
                        subscription_id TEXT,
                        group_name TEXT DEFAULT ''
                    );
                    CREATE TABLE IF NOT EXISTS rules (
                        id TEXT PRIMARY KEY,
                        rule_type TEXT NOT NULL,
                        value TEXT NOT NULL,
                        node_id TEXT,
                        priority INTEGER DEFAULT 0,
                        enabled INTEGER DEFAULT 1,
                        created_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS settings (
                        key TEXT PRIMARY KEY,
                        value TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS speed_history (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        node_id TEXT NOT NULL,
                        latency_ms REAL,
                        download_mbps REAL,
                        upload_mbps REAL,
                        error_rate REAL,
                        tested_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS subscriptions (
                        id TEXT PRIMARY KEY,
                        url TEXT NOT NULL,
                        name TEXT NOT NULL DEFAULT '',
                        group_name TEXT DEFAULT '',
                        interval_mins INTEGER DEFAULT 60,
                        enabled INTEGER DEFAULT 1,
                        created_at TEXT NOT NULL
                    );
                    CREATE TABLE IF NOT EXISTS traffic_log (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        timestamp TEXT NOT NULL,
                        level TEXT NOT NULL,
                        message TEXT NOT NULL
                    );
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'dark');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('autostart', 'false');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('auto_switch', 'true');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('latency_threshold_ms', '2000');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('error_threshold_pct', '30');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('subscription_url', '');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('subscription_interval_mins', '60');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('log_level', 'info');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('portable_mode', 'false');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('bypass_ru', 'true');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('bypass_local', 'true');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('kill_switch', 'false');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('dns_server', '1.1.1.1');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('proxy_mode', 'system');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('auto_ping', 'true');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('ping_interval', '300');
                    INSERT OR IGNORE INTO settings (key, value) VALUES ('split_processes', '');
                    ",
                )
                .map_err(|e| error!("Failed to init DB: {e}"))
                .ok();

                // Migrate old databases: add columns that may not exist
                let migrations = [
                    "ALTER TABLE nodes ADD COLUMN subscription_id TEXT",
                    "ALTER TABLE nodes ADD COLUMN group_name TEXT DEFAULT ''",
                    "ALTER TABLE subscriptions ADD COLUMN group_name TEXT DEFAULT ''",
                    "ALTER TABLE subscriptions ADD COLUMN name TEXT NOT NULL DEFAULT ''",
                    "ALTER TABLE subscriptions ADD COLUMN interval_mins INTEGER DEFAULT 60",
                ];
                for sql in &migrations {
                    conn.execute(sql, []).ok();
                }

                self.db = Some(Arc::new(Mutex::new(conn)));
                info!("Database initialized successfully");
            }
            Err(e) => {
                error!("Failed to open database: {e}");
            }
        }
    }

    fn get_setting_bool(settings: &Value, key: &str, default: bool) -> bool {
        settings
            .get(key)
            .and_then(|v| v.as_bool().or_else(|| v.as_str().and_then(|s| s.parse::<bool>().ok())))
            .unwrap_or(default)
    }

    fn get_setting_u64(settings: &Value, key: &str, default: u64) -> u64 {
        settings
            .get(key)
            .and_then(|v| v.as_u64().or_else(|| v.as_i64().and_then(|n| u64::try_from(n).ok())).or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok())))
            .unwrap_or(default)
    }

    fn get_setting_f64(settings: &Value, key: &str, default: f64) -> f64 {
        settings
            .get(key)
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)).or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok())))
            .unwrap_or(default)
    }

    pub async fn connect(&mut self, node_id: &str) -> Result<(), String> {
        if self.status.connected {
            self.disconnect().await?;
        }

        // Wait for port to be released after disconnect
        tokio::time::sleep(Duration::from_millis(300)).await;

        let node = self.get_node_by_id(node_id).await?;
        let settings = self.get_settings().await.unwrap_or_default();
        let connection_timeout = Self::get_setting_u64(&settings, "connection_timeout", 20);
        let max_retries = Self::get_setting_u64(&settings, "max_retries", 3).clamp(1, 10) as u32;

        let config = config_gen::generate_config(&node, &self.get_rules_inner().await?, &settings);
        let config_path = self.config_dir.join("config.json");
        let config_str = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {e}"))?;
        std::fs::write(&config_path, &config_str)
            .map_err(|e| format!("Failed to write config: {e}"))?;

        if !self.sing_box_path.exists() {
            return Err(format!(
                "sing-box binary not found at {:?}. Please ensure it's downloaded.",
                self.sing_box_path
            ));
        }

        // Copy geo databases to config_dir so sing-box can find them
        if let Some((geoip_src, geosite_src)) = self.get_geo_flags() {
            let geoip_dst = self.config_dir.join("geoip.db");
            let geosite_dst = self.config_dir.join("geosite.db");
            // Always overwrite to ensure we have the correct version
            let _ = std::fs::copy(&geoip_src, &geoip_dst);
            let _ = std::fs::copy(&geosite_src, &geosite_dst);
        }

        let mut cmd = Command::new(&self.sing_box_path);
        let config_arg = config_path
            .to_str()
            .ok_or_else(|| "Config path contains invalid UTF-8".to_string())?;
        cmd.args(["run", "-c", config_arg]);
        cmd.current_dir(&self.config_dir); // keep cache.db etc. outside project tree
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped());

        cmd.stderr(Stdio::piped());
        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to start sing-box: {e}"))?;

        // Capture stderr in a background thread for diagnostics if something goes wrong
        let child_stderr = child.stderr.take();
        let stderr_dump = Arc::new(std::sync::Mutex::new(String::new()));
        let stderr_dump_clone = stderr_dump.clone();
        if let Some(stderr) = child_stderr {
            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let mut reader = stderr;
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            if let Ok(s) = String::from_utf8(buf[..n].to_vec()) {
                                let mut dump = stderr_dump_clone.lock().unwrap();
                                dump.push_str(&s);
                                if dump.len() > 16384 {
                                    dump.truncate(16384);
                                }
                            }
                        }
                    }
                }
            });
        }

        // Wait and verify sing-box is alive + proxy port is up before enabling system proxy
        let proxy_ready = wait_for_proxy::wait(
            "127.0.0.1:2080",
            max_retries,
            (connection_timeout.saturating_mul(1000) / u64::from(max_retries)).max(250),
            Some(&mut child),
        ).await;

        if !proxy_ready {
            let _ = child.kill();
            let _ = child.wait();
            let stderr = stderr_dump.lock().unwrap().clone();
            let stderr_trimmed: Vec<&str> = stderr.lines().take(10).collect();
            let stderr_info = if stderr_trimmed.is_empty() {
                String::new()
            } else {
                format!("\n  stderr: {}", stderr_trimmed.join("\n  "))
            };
            let reason = format!("Connection failed — proxy not ready{}", stderr_info);
            self.add_log("error", &reason);
            return Err(reason);
        }

        self.process = Some(child);
        self.status.connected = true;

        // Only set system proxy in proxy mode, not tunnel mode
        let is_tunnel = settings.get("proxy_mode").and_then(|v| v.as_str()) == Some("tunnel");
        #[cfg(target_os = "windows")]
        if !is_tunnel {
            self.enable_system_proxy();
        }

        self.status.current_node_id = Some(node_id.to_string());
        self.status.current_node_name = node.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
        self.status.connected_at = Some(Utc::now().timestamp());

        self.add_log("info", &format!("Connected to node: {}", node_id));

        // Start monitoring thread
        if let Some(child) = self.process.as_mut() {
            let stdout = child.stdout.take()
                .ok_or_else(|| "Failed to capture stdout".to_string())?;

            let app = self.app.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let _ = app.emit("sing-box-log", &line);
                        if let Ok(log_entry) = serde_json::from_str::<Value>(&line) {
                            if let Some(level) = log_entry.get("level").and_then(|v| v.as_str()) {
                                if let Some(msg) = log_entry.get("message").and_then(|v| v.as_str()) {
                                    let _ = app.emit("vpn-log", json!({
                                        "level": level,
                                        "message": msg,
                                        "timestamp": Utc::now().to_rfc3339(),
                                    }));
                                }
                            }
                        }
                    }
                }
            });
        }

        // Start auto-switch background task
        self.start_auto_switch();

        self.emit_status();
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), String> {
        // Stop auto-switch task
        self.stop_auto_switch();

        // ALWAYS disable system proxy on disconnect — prevents stale proxy on crash/exit
        #[cfg(target_os = "windows")]
        self.disable_system_proxy();

        if let Some(mut child) = self.process.take() {
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("taskkill")
                    .args(["/PID", &child.id().to_string(), "/F"])
                    .output();
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = nix::sys::signal::kill(
                    nix::unistd::Pid::from_raw(child.id() as i32),
                    nix::sys::signal::SIGTERM,
                );
            }

            let _ = child.wait();
        }

        self.status = VpnStatus {
            connected: false,
            current_node_id: None,
            current_node_name: None,
            connected_at: None,
            download_speed: 0.0,
            upload_speed: 0.0,
            total_download: 0,
            total_upload: 0,
            latency_ms: None,
        };

        self.add_log("info", "Disconnected from VPN");
        self.emit_status();
        Ok(())
    }

    /// Get the paths to geoip.db and geosite.db if they exist and look valid
    fn get_geo_flags(&self) -> Option<(PathBuf, PathBuf)> {
        let resource_dir = self.app.path().resource_dir().ok()?;
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let try_dirs = [&resource_dir, &manifest_dir];
        for dir in &try_dirs {
            let geoip = dir.join("geo").join("geoip.db");
            let geosite = dir.join("geo").join("geosite.db");
            // sing-box geo databases are typically 3-8 MB; reject tiny files (404 pages, errors)
            let geoip_ok = geoip.exists() && geoip.metadata().map(|m| m.len()).unwrap_or(0) > 100_000;
            let geosite_ok = geosite.exists() && geosite.metadata().map(|m| m.len()).unwrap_or(0) > 100_000;
            if geoip_ok && geosite_ok {
                return Some((geoip, geosite));
            }
            if geoip.exists() && !geoip_ok {
                warn!("geoip.db exists but is too small ({} bytes), treating as missing", geoip.metadata().map(|m| m.len()).unwrap_or(0));
            }
            if geosite.exists() && !geosite_ok {
                warn!("geosite.db exists but is too small ({} bytes), treating as missing", geosite.metadata().map(|m| m.len()).unwrap_or(0));
            }
        }

        warn!(
            "Valid geo databases not found at {:?}/geo/ or {:?}/geo/",
            resource_dir, manifest_dir
        );
        None
    }

    pub async fn verify_connection(&mut self) -> Result<bool, String> {
        if !self.status.connected {
            return Err("Not connected".to_string());
        }

        let settings = self.get_settings().await.unwrap_or_default();
        let connection_timeout = Self::get_setting_u64(&settings, "connection_timeout", 20).clamp(5, 60);
        let max_retries = Self::get_setting_u64(&settings, "max_retries", 3).clamp(1, 10) as u32;

        // Wait for the mixed proxy to be listening (retry up to 7.5s)
        self.add_log("info", "Waiting for proxy on 127.0.0.1:2080...");
        if !wait_for_proxy::wait(
            "127.0.0.1:2080",
            max_retries,
            (connection_timeout.saturating_mul(1000) / u64::from(max_retries)).max(250),
            None,
        ).await {
            self.add_log("error", "Proxy on 127.0.0.1:2080 never became ready");
            return Ok(false);
        }
        self.add_log("info", "Proxy is listening, verifying connection via VPN...");

        // Verify through the mixed proxy (127.0.0.1:2080)
        let client = reqwest::Client::builder()
            .proxy(reqwest::Proxy::http("http://127.0.0.1:2080").map_err(|e| format!("Proxy: {e}"))?)
            .proxy(reqwest::Proxy::https("http://127.0.0.1:2080").map_err(|e| format!("Proxy: {e}"))?)
            .timeout(Duration::from_secs(connection_timeout))
            .build()
            .map_err(|e| format!("HTTP client error: {e}"))?;

        // Check known globally reachable URLs through the VPN
        let urls = [
            "https://www.gstatic.com/generate_204",
            "https://www.google.com/generate_204",
            "https://cloudflare.com/cdn-cgi/trace",
        ];

        let mut basic_ok = false;
        for url in &urls {
            match client.get(*url).send().await {
                Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 204 => {
                    self.add_log("info", &format!("Connection verified via {url}"));
                    basic_ok = true;
                    break;
                }
                Ok(resp) => {
                    self.add_log("warn", &format!("Verify {url} returned {}", resp.status()));
                }
                Err(e) => {
                    self.add_log("warn", &format!("Verify {url} failed: {e}"));
                }
            }
        }

        if !basic_ok {
            self.add_log("error", "Connection verification failed — basic URLs unreachable");
            return Ok(false);
        }

        // Additional check: try instagram.com (blocked in Russia)
        // If it works through VPN → server truly works (not a local/RU server)
        match client.get("https://www.instagram.com/").send().await {
            Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 301 || resp.status().as_u16() == 302 => {
                self.add_log("info", "Instagram accessible — VPN fully working");
                Ok(true)
            }
            Ok(resp) => {
                self.add_log("warn", &format!("Instagram returned {} — VPN may not bypass blocks", resp.status()));
                Ok(true)
            }
            Err(e) => {
                self.add_log("warn", &format!("Instagram check failed: {e} — VPN connected but may not bypass blocks"));
                Ok(true)
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn enable_system_proxy(&mut self) {
        let _ = Command::new("reg")
            .args([
                "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "1", "/f",
            ])
            .output();
        let _ = Command::new("reg")
            .args([
                "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v", "ProxyServer", "/d", "127.0.0.1:2080", "/f",
            ])
            .output();
        let _ = Command::new("netsh")
            .args(["winhttp", "set", "proxy", "127.0.0.1:2080"])
            .output();
        self.add_log("info", "System proxy set to 127.0.0.1:2080");
    }

    #[cfg(target_os = "windows")]
    fn disable_system_proxy(&mut self) {
        let _ = Command::new("reg")
            .args([
                "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "0", "/f",
            ])
            .output();
        let _ = Command::new("netsh")
            .args(["winhttp", "reset", "proxy"])
            .output();
        self.add_log("info", "System proxy cleared");
    }

    /// Force cleanup: kill sing-box and disable proxy. Called on app exit/crash.
    pub fn force_cleanup(&mut self) {
        // Kill sing-box process
        if let Some(mut child) = self.process.take() {
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("taskkill")
                    .args(["/PID", &child.id().to_string(), "/F"])
                    .output();
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = nix::sys::signal::kill(
                    nix::unistd::Pid::from_raw(child.id() as i32),
                    nix::sys::signal::SIGTERM,
                );
            }
            let _ = child.wait();
        }
        // Always disable proxy
        #[cfg(target_os = "windows")]
        self.disable_system_proxy();
        // Stop background tasks
        self.stop_auto_switch();
        self.stop_subscription_refresh();
        self.stop_auto_ping();
    }

    pub fn get_status(&self) -> Result<Value, String> {
        Ok(serde_json::to_value(&self.status).map_err(|e| format!("Serialize error: {e}"))?)
    }

    pub async fn switch_node(&mut self, node_id: &str) -> Result<(), String> {
        if self.status.connected {
            self.disconnect().await?;
        }
        self.connect(node_id).await
    }

    pub async fn get_nodes(&self) -> Result<Vec<Value>, String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            let mut stmt = conn
                .prepare("SELECT id, name, protocol, server, port, config, latency_ms, uptime, error_count, last_test_at, enabled, subscription_id, group_name FROM nodes ORDER BY group_name, name")
                .map_err(|e| format!("DB error: {e}"))?;

            let rows = stmt
                .query_map([], |row| {
                    let config_str: String = row.get(5)?;
                    let config: Value = serde_json::from_str(&config_str).unwrap_or(json!({}));
                    Ok(json!({
                        "id": row.get::<_, String>(0)?,
                        "name": row.get::<_, String>(1)?,
                        "protocol": row.get::<_, String>(2)?,
                        "server": row.get::<_, String>(3)?,
                        "port": row.get::<_, i64>(4)?,
                        "config": config,
                        "latency_ms": row.get::<_, Option<f64>>(6)?,
                        "uptime": row.get::<_, Option<f64>>(7)?,
                        "error_count": row.get::<_, i64>(8)?,
                        "last_test_at": row.get::<_, Option<String>>(9)?,
                        "enabled": row.get::<_, i64>(10)? != 0,
                        "subscription_id": row.get::<_, Option<String>>(11)?,
                        "group_name": row.get::<_, Option<String>>(12)?.unwrap_or_default(),
                    }))
                })
                .map_err(|e| format!("Query error: {e}"))?;

            let nodes: Vec<Value> = rows.filter_map(|r| r.ok()).collect();
            Ok(nodes)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_node_by_id(&self, node_id: &str) -> Result<Value, String> {
        let nodes = self.get_nodes().await?;
        nodes
            .into_iter()
            .find(|n| n.get("id").and_then(|v| v.as_str()) == Some(node_id))
            .ok_or_else(|| format!("Node not found: {node_id}"))
    }

    pub async fn add_node(&self, node: Value) -> Result<(), String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            let id = uuid::Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            let config_str = serde_json::to_string(&node.get("config").unwrap_or(&json!({})))
                .map_err(|e| format!("Serialize error: {e}"))?;

            conn.execute(
                "INSERT INTO nodes (id, name, protocol, server, port, config, created_at, updated_at, enabled, subscription_id, group_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?10)",
                rusqlite::params![
                    id,
                    node.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown"),
                    node.get("protocol").and_then(|v| v.as_str()).unwrap_or("unknown"),
                    node.get("server").and_then(|v| v.as_str()).unwrap_or(""),
                    node.get("port").and_then(|v| v.as_i64()).unwrap_or(0),
                    config_str,
                    now,
                    now,
                    node.get("subscription_id").and_then(|v| v.as_str()),
                    node.get("group_name").and_then(|v| v.as_str()).unwrap_or(""),
                ],
            )
            .map_err(|e| format!("DB insert error: {e}"))?;
            Ok(())
        } else {
            Err("Database not initialized".to_string())
        }
    }

    pub async fn delete_node(&self, node_id: &str) -> Result<(), String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            conn.execute("DELETE FROM nodes WHERE id = ?1", rusqlite::params![node_id])
                .map_err(|e| format!("DB delete error: {e}"))?;
            Ok(())
        } else {
            Err("Database not initialized".to_string())
        }
    }

    pub async fn import_subscription(&self, url: &str, group_name: &str) -> Result<Vec<Value>, String> {
        // Create a subscription record so nodes get proper subscription_id
        let sub_id = if !group_name.is_empty() || !url.is_empty() {
            let sub_name = if !group_name.is_empty() { group_name.to_string() } else { url.to_string() };
            let sub = self.add_subscription(url.to_string(), sub_name, group_name.to_string(), 60).await?;
            sub.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
        } else {
            None
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("HTTP client error: {e}"))?;

        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {e}"))?;

        let decoded = if let Ok(bytes) = base64_decode(&text) {
            String::from_utf8(bytes).unwrap_or(text.clone())
        } else {
            text
        };

        let nodes: Vec<Value> = if let Ok(arr) = serde_json::from_str::<Vec<Value>>(&decoded) {
            arr
        } else {
            let mut nodes = Vec::new();
            for line in decoded.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    if let Ok(node) = parse_share_link(line) {
                        nodes.push(node);
                    }
                }
            }
            nodes
        };

        let mut imported = Vec::new();
        for mut node in nodes {
            if let Some(obj) = node.as_object_mut() {
                if let Some(ref sid) = sub_id {
                    obj.insert("subscription_id".to_string(), json!(sid));
                }
                if !group_name.is_empty() {
                    obj.insert("group_name".to_string(), json!(group_name));
                }
            }
            self.add_node(node.clone()).await?;
            imported.push(node);
        }

        Ok(imported)
    }

    pub async fn get_rules_inner(&self) -> Result<Vec<Value>, String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            let mut stmt = conn
                .prepare("SELECT id, rule_type, value, node_id, priority, enabled FROM rules ORDER BY priority DESC")
                .map_err(|e| format!("DB error: {e}"))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>(0)?,
                        "rule_type": row.get::<_, String>(1)?,
                        "value": row.get::<_, String>(2)?,
                        "node_id": row.get::<_, Option<String>>(3)?,
                        "priority": row.get::<_, i64>(4)?,
                        "enabled": row.get::<_, i64>(5)? != 0,
                    }))
                })
                .map_err(|e| format!("Query error: {e}"))?;

            Ok(rows.filter_map(|r| r.ok()).collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn get_rules(&self) -> Result<Vec<Value>, String> {
        self.get_rules_inner().await
    }

    pub async fn add_rule(&self, rule: Value) -> Result<(), String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            let id = uuid::Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();

            conn.execute(
                "INSERT INTO rules (id, rule_type, value, node_id, priority, enabled, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    id,
                    rule.get("rule_type").and_then(|v| v.as_str()).unwrap_or("domain"),
                    rule.get("value").and_then(|v| v.as_str()).unwrap_or(""),
                    rule.get("node_id").and_then(|v| v.as_str()),
                    rule.get("priority").and_then(|v| v.as_i64()).unwrap_or(0),
                    rule.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                    now,
                ],
            )
            .map_err(|e| format!("DB insert error: {e}"))?;
            Ok(())
        } else {
            Err("Database not initialized".to_string())
        }
    }

    pub async fn update_rule(&self, rule_id: &str, rule: Value) -> Result<(), String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            conn.execute(
                "UPDATE rules SET rule_type = ?1, value = ?2, node_id = ?3, priority = ?4, enabled = ?5 WHERE id = ?6",
                rusqlite::params![
                    rule.get("rule_type").and_then(|v| v.as_str()).unwrap_or("domain"),
                    rule.get("value").and_then(|v| v.as_str()).unwrap_or(""),
                    rule.get("node_id").and_then(|v| v.as_str()),
                    rule.get("priority").and_then(|v| v.as_i64()).unwrap_or(0),
                    rule.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                    rule_id,
                ],
            )
            .map_err(|e| format!("DB update error: {e}"))?;
            Ok(())
        } else {
            Err("Database not initialized".to_string())
        }
    }

    pub async fn delete_rule(&self, rule_id: &str) -> Result<(), String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            conn.execute("DELETE FROM rules WHERE id = ?1", rusqlite::params![rule_id])
                .map_err(|e| format!("DB delete error: {e}"))?;
            Ok(())
        } else {
            Err("Database not initialized".to_string())
        }
    }

    pub async fn reorder_rules(&self, rule_ids: Vec<String>) -> Result<(), String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            for (i, rule_id) in rule_ids.iter().enumerate() {
                let priority = (rule_ids.len() - i) as i64;
                conn.execute(
                    "UPDATE rules SET priority = ?1 WHERE id = ?2",
                    rusqlite::params![priority, rule_id],
                )
                .map_err(|e| format!("DB reorder error: {e}"))?;
            }
            Ok(())
        } else {
            Err("Database not initialized".to_string())
        }
    }

    pub async fn test_rule(&self, domain: &str, geo_db: Option<&GeoDatabase>) -> Result<Value, String> {
        let rules = self.get_rules_inner().await?;
        let node = crate::rules_engine::evaluate_domain(domain, &rules, geo_db).await?;
        Ok(node)
    }

    pub async fn get_settings(&self) -> Result<Value, String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            let mut stmt = conn
                .prepare("SELECT key, value FROM settings")
                .map_err(|e| format!("DB error: {e}"))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|e| format!("Query error: {e}"))?;

            let mut map = serde_json::Map::new();
            for row in rows {
                if let Ok((key, value)) = row {
                    if let Ok(v) = serde_json::from_str::<Value>(&value) {
                        map.insert(key, v);
                    } else {
                        // Handle string booleans: "true"/"false" → JSON bool
                        let v = match value.as_str() {
                            "true" => Value::Bool(true),
                            "false" => Value::Bool(false),
                            _ => {
                                // Try parse as number
                                if let Ok(n) = value.parse::<i64>() {
                                    Value::Number(n.into())
                                } else if let Ok(n) = value.parse::<f64>() {
                                    serde_json::Number::from_f64(n).map(Value::Number).unwrap_or(Value::String(value))
                                } else {
                                    Value::String(value)
                                }
                            }
                        };
                        map.insert(key, v);
                    }
                }
            }
            Ok(Value::Object(map))
        } else {
            Ok(json!({}))
        }
    }

    pub async fn update_settings(&self, settings: Value) -> Result<(), String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            if let Some(obj) = settings.as_object() {
                for (key, value) in obj {
                    let val_str = match value {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    // Use INSERT OR REPLACE to handle keys that don't exist yet
                    conn.execute(
                        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
                        rusqlite::params![key, val_str],
                    )
                    .map_err(|e| format!("DB update error: {e}"))?;
                }
            }
            Ok(())
        } else {
            Err("Database not initialized".to_string())
        }
    }

    pub async fn replace_settings_snapshot(&self, data: Value) -> Result<(), String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            conn.execute("DELETE FROM nodes", []).map_err(|e| format!("DB reset error: {e}"))?;
            conn.execute("DELETE FROM rules", []).map_err(|e| format!("DB reset error: {e}"))?;
            conn.execute("DELETE FROM subscriptions", []).map_err(|e| format!("DB reset error: {e}"))?;

            if let Some(settings) = data.get("settings") {
                drop(conn);
                self.update_settings(settings.clone()).await?;
            } else {
                drop(conn);
            }

            if let Some(db) = &self.db {
                let conn = db.lock().await;

                if let Some(subs) = data.get("subscriptions").and_then(|v| v.as_array()) {
                    for sub in subs {
                        let id = sub.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let url = sub.get("url").and_then(|v| v.as_str()).unwrap_or("");
                        if id.is_empty() || url.is_empty() {
                            continue;
                        }
                        conn.execute(
                            "INSERT INTO subscriptions (id, url, name, group_name, interval_mins, enabled, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            rusqlite::params![
                                id,
                                url,
                                sub.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                                sub.get("group_name").and_then(|v| v.as_str()).unwrap_or(""),
                                sub.get("interval_mins").and_then(|v| v.as_i64()).unwrap_or(60),
                                sub.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                                Utc::now().to_rfc3339(),
                            ],
                        ).map_err(|e| format!("Subscription restore error: {e}"))?;
                    }
                }

                if let Some(nodes) = data.get("nodes").and_then(|v| v.as_array()) {
                    for node in nodes {
                        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = node.get("name").and_then(|v| v.as_str()).unwrap_or("").trim();
                        let protocol = node.get("protocol").and_then(|v| v.as_str()).unwrap_or("").trim();
                        let server = node.get("server").and_then(|v| v.as_str()).unwrap_or("").trim();
                        let port = node.get("port").and_then(|v| v.as_i64()).unwrap_or(0);
                        if id.is_empty() || name.is_empty() || protocol.is_empty() || server.is_empty() || port <= 0 {
                            continue;
                        }
                        let config_str = serde_json::to_string(&node.get("config").cloned().unwrap_or_else(|| json!({})))
                            .map_err(|e| format!("Node config restore error: {e}"))?;
                        conn.execute(
                            "INSERT INTO nodes (id, name, protocol, server, port, config, latency_ms, uptime, error_count, last_test_at, created_at, updated_at, enabled, subscription_id, group_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                            rusqlite::params![
                                id,
                                name,
                                protocol,
                                server,
                                port,
                                config_str,
                                node.get("latency_ms").and_then(|v| v.as_f64()),
                                node.get("uptime").and_then(|v| v.as_f64()),
                                node.get("error_count").and_then(|v| v.as_i64()).unwrap_or(0),
                                node.get("last_test_at").and_then(|v| v.as_str()),
                                Utc::now().to_rfc3339(),
                                Utc::now().to_rfc3339(),
                                node.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                                node.get("subscription_id").and_then(|v| v.as_str()),
                                node.get("group_name").and_then(|v| v.as_str()).unwrap_or(""),
                            ],
                        ).map_err(|e| format!("Node restore error: {e}"))?;
                    }
                }

                if let Some(rules) = data.get("rules").and_then(|v| v.as_array()) {
                    for rule in rules {
                        let id = rule.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let value = rule.get("value").and_then(|v| v.as_str()).unwrap_or("").trim();
                        if id.is_empty() || value.is_empty() {
                            continue;
                        }
                        conn.execute(
                            "INSERT INTO rules (id, rule_type, value, node_id, priority, enabled, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            rusqlite::params![
                                id,
                                rule.get("rule_type").and_then(|v| v.as_str()).unwrap_or("domain_suffix"),
                                value,
                                rule.get("node_id").and_then(|v| v.as_str()),
                                rule.get("priority").and_then(|v| v.as_i64()).unwrap_or(0),
                                rule.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                                Utc::now().to_rfc3339(),
                            ],
                        ).map_err(|e| format!("Rule restore error: {e}"))?;
                    }
                }
            }
            Ok(())
        } else {
            Err("Database not initialized".to_string())
        }
    }

    pub async fn export_logs(&self, path: &str) -> Result<(), String> {
        let logs_json = serde_json::to_string_pretty(&self.logs)
            .map_err(|e| format!("Serialize error: {e}"))?;
        std::fs::write(path, &logs_json)
            .map_err(|e| format!("Failed to write log file: {e}"))?;
        Ok(())
    }

    pub fn emit_status(&self) {
        let _ = self.app.emit("vpn-status", json!({
            "connected": self.status.connected,
            "current_node_id": self.status.current_node_id,
            "current_node_name": self.status.current_node_name,
            "connected_at": self.status.connected_at,
            "download_speed": self.status.download_speed,
            "upload_speed": self.status.upload_speed,
            "total_download": self.status.total_download,
            "total_upload": self.status.total_upload,
            "latency_ms": self.status.latency_ms,
        }));
    }

    fn add_log(&mut self, level: &str, message: &str) {
        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: level.to_string(),
            message: message.to_string(),
        };
        self.logs.push(entry);
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
        let _ = self.app.emit("vpn-log", json!({
            "level": level,
            "message": message,
            "timestamp": Utc::now().to_rfc3339(),
        }));
    }

    // ============ Auto-Switch ============

    /// Start background auto-switch task that monitors latency and switches if needed
    fn start_auto_switch(&mut self) {
        self.stop_auto_switch();

        let self_weak = self.self_ref.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(15)).await;

            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;

                // Access manager via weak reference
                let manager_arc = match self_weak.as_ref().and_then(|w| w.upgrade()) {
                    Some(arc) => arc,
                    None => break,
                };
                let mgr = manager_arc.lock().await;

                if !mgr.status.connected {
                    break;
                }

                let settings = mgr.get_settings().await.unwrap_or_default();
                let auto_switch = Self::get_setting_bool(&settings, "auto_switch", true);

                if !auto_switch {
                    continue;
                }

                let threshold = Self::get_setting_f64(&settings, "latency_threshold_ms", 2000.0);

                // Test current node latency
                let current_node_id = match &mgr.status.current_node_id {
                    Some(id) => id.clone(),
                    None => continue,
                };

                // Release lock before doing network test to avoid holding it
                drop(mgr);

                let start = std::time::Instant::now();
                let current_latency = tokio::time::timeout(
                    Duration::from_secs(10),
                    tokio::net::TcpStream::connect(format!("8.8.8.8:53")),
                )
                .await
                .ok()
                .and_then(|r| r.ok())
                .map(|_| start.elapsed().as_secs_f64() * 1000.0)
                .unwrap_or(f64::MAX);

                let mut mgr = manager_arc.lock().await;

                if !mgr.status.connected {
                    break;
                }

                if current_latency < f64::MAX {
                    mgr.status.latency_ms = Some(current_latency);
                    mgr.emit_status();
                }

                // Check if threshold exceeded
                if current_latency > threshold {
                    info!("Latency {current_latency:.0}ms exceeds threshold {threshold:.0}ms. Searching for better node...");

                    // Find the best node (lowest latency)
                    let nodes = mgr.get_nodes().await.unwrap_or_default();

                    // We need to test other nodes - spawn subtasks
                    // For simplicity, find a node with lower recorded latency
                    let current_id = &current_node_id;
                    let best_node = nodes.iter()
                        .filter(|n| {
                            n.get("id").and_then(|v| v.as_str()) != Some(current_id.as_str())
                                && n.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true)
                        })
                        .filter_map(|n| {
                            let id = n.get("id")?.as_str()?.to_string();
                            let lat = n.get("latency_ms")?.as_f64()?;
                            if lat > 0.0 && lat < threshold * 0.7 {
                                Some((id, lat))
                            } else {
                                None
                            }
                        })
                        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

                    if let Some((node_id, latency)) = best_node {
                        info!("Auto-switching to {node_id} (latency: {latency:.0}ms)");
                        mgr.add_log("info", &format!("Auto-switch: latency {current_latency:.0}ms > threshold {threshold:.0}ms, switching to node {node_id}"));

                        // Disconnect and connect to new node
                        drop(mgr);
                        let mut mgr = manager_arc.lock().await;
                        if mgr.status.connected {
                            let _ = mgr.disconnect().await;
                            let _ = mgr.connect(&node_id).await;
                        }
                        // The new connection will start its own auto-switch task
                        break;
                    } else {
                        warn!("No better node found for auto-switch");
                    }
                }
            }
        });

        self.auto_switch_task = Some(handle);
    }

    fn stop_auto_switch(&mut self) {
        if let Some(handle) = self.auto_switch_task.take() {
            handle.abort();
        }
    }

    // ============ Subscription CRUD ============

    pub async fn get_subscriptions(&self) -> Result<Vec<Value>, String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            let mut stmt = conn
                .prepare("SELECT id, url, name, group_name, interval_mins, enabled FROM subscriptions ORDER BY name")
                .map_err(|e| format!("DB error: {e}"))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(json!({
                        "id": row.get::<_, String>(0)?,
                        "url": row.get::<_, String>(1)?,
                        "name": row.get::<_, String>(2)?,
                        "group_name": row.get::<_, String>(3)?,
                        "interval_mins": row.get::<_, i64>(4)?,
                        "enabled": row.get::<_, i64>(5)? != 0,
                    }))
                })
                .map_err(|e| format!("Query error: {e}"))?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn add_subscription(&self, url: String, name: String, group_name: String, interval_mins: i64) -> Result<Value, String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            let id = uuid::Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO subscriptions (id, url, name, group_name, interval_mins, enabled, created_at) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
                rusqlite::params![id, url, name, group_name, interval_mins, now],
            )
            .map_err(|e| format!("DB insert error: {e}"))?;
            Ok(json!({ "id": id }))
        } else {
            Err("Database not initialized".to_string())
        }
    }

    pub async fn delete_subscription(&self, sub_id: &str) -> Result<(), String> {
        if let Some(db) = &self.db {
            let conn = db.lock().await;
            conn.execute("DELETE FROM subscriptions WHERE id = ?1", rusqlite::params![sub_id])
                .map_err(|e| format!("DB delete error: {e}"))?;
            // Unlink nodes from this subscription
            conn.execute("UPDATE nodes SET subscription_id = NULL WHERE subscription_id = ?1", rusqlite::params![sub_id])
                .map_err(|e| format!("DB update error: {e}"))?;
            Ok(())
        } else {
            Err("Database not initialized".to_string())
        }
    }

    pub async fn import_subscription_url(&mut self, sub_id: &str) -> Result<Vec<Value>, String> {
        // Get subscription URL from DB
        let subs = self.get_subscriptions().await?;
        let sub = subs.into_iter().find(|s| s.get("id").and_then(|v| v.as_str()) == Some(sub_id))
            .ok_or_else(|| "Subscription not found".to_string())?;

        let url = sub.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let group = sub.get("group_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let name = sub.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();

        if url.is_empty() {
            return Err("Empty subscription URL".to_string());
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("HTTP client error: {e}"))?;

        let resp = client.get(&url).send().await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        let text = resp.text().await
            .map_err(|e| format!("Failed to read response: {e}"))?;

        let decoded = if let Ok(bytes) = base64_decode(&text) {
            String::from_utf8(bytes).unwrap_or(text.clone())
        } else {
            text
        };

        let nodes: Vec<Value> = if let Ok(arr) = serde_json::from_str::<Vec<Value>>(&decoded) {
            arr
        } else {
            let mut nodes = Vec::new();
            for line in decoded.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    if let Ok(node) = parse_share_link(line) {
                        nodes.push(node);
                    }
                }
            }
            nodes
        };

        let mut imported = Vec::new();
        for mut node in nodes {
            // Tag with subscription group info
            if let Some(obj) = node.as_object_mut() {
                obj.insert("subscription_id".to_string(), json!(sub_id));
                if !group.is_empty() {
                    obj.insert("group_name".to_string(), json!(&group));
                }
            }
            self.add_node(node.clone()).await?;
            imported.push(node);
        }

        self.add_log("info", &format!("Imported {} nodes from subscription '{}'", imported.len(), name));
        Ok(imported)
    }

    // ============ Subscription Refresh ============

    /// Start background subscription refresh task
    pub fn start_subscription_refresh(&mut self) {
        self.stop_subscription_refresh();

        let app = self.app.clone();
        let self_weak = self.self_ref.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(30)).await;

            loop {
                let manager_arc = match self_weak.as_ref().and_then(|w| w.upgrade()) {
                    Some(arc) => arc,
                    None => break,
                };
                let mgr = manager_arc.lock().await;

                let settings = mgr.get_settings().await.unwrap_or_default();
                let url = settings
                    .get("subscription_url")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                if url.is_empty() {
                    drop(mgr);
                    tokio::time::sleep(Duration::from_secs(300)).await;
                    continue;
                }

                let interval_mins = settings
                    .get("subscription_interval_mins")
                    .and_then(|v| {
                        v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
                    })
                    .unwrap_or(60);

                let interval = Duration::from_secs(interval_mins * 60);

                drop(mgr);

                info!("Refreshing subscription from: {url}");

                // Refresh subscription inline using weak ref
                let _sub_result = refresh_subscription_inner(&app, &url).await;

                tokio::time::sleep(interval).await;
            }
        });

        self.subscription_refresh_task = Some(handle);
    }

    fn stop_subscription_refresh(&mut self) {
        if let Some(handle) = self.subscription_refresh_task.take() {
            handle.abort();
        }
    }

    // ============ Auto-Ping ============

    pub fn start_auto_ping(&mut self) {
        self.stop_auto_ping();

        let app = self.app.clone();
        let self_weak = self.self_ref.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(30)).await;

            loop {
                // Check settings for auto_ping enabled and interval
                let (auto_ping, interval) = {
                    let manager_arc = match self_weak.as_ref().and_then(|w| w.upgrade()) {
                        Some(arc) => arc,
                        None => break,
                    };
                    let mgr = manager_arc.lock().await;
                    let settings = mgr.get_settings().await.unwrap_or_default();
                    let enabled = Self::get_setting_bool(&settings, "auto_ping", true);
                    let interval = Self::get_setting_u64(&settings, "ping_interval", 300);
                    (enabled, interval)
                };

                if !auto_ping {
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    continue;
                }

                // Test all nodes latency
                let node_infos = {
                    let manager_arc = match self_weak.as_ref().and_then(|w| w.upgrade()) {
                        Some(arc) => arc,
                        None => break,
                    };
                    let mgr = manager_arc.lock().await;
                    match mgr.get_nodes().await {
                        Ok(nodes) => {
                            let mut infos = Vec::new();
                            for n in &nodes {
                                if let (Some(id), Some(server), Some(port)) = (
                                    n.get("id").and_then(|v| v.as_str()),
                                    n.get("server").and_then(|v| v.as_str()),
                                    n.get("port").and_then(|v| v.as_u64()),
                                ) {
                                    infos.push((id.to_string(), server.to_string(), port as u16));
                                }
                            }
                            infos
                        }
                        Err(_) => continue,
                    }
                };

                for (node_id, server, port) in &node_infos {
                    let start = std::time::Instant::now();
                    let connect_result = tokio::time::timeout(
                        Duration::from_secs(5),
                        tokio::net::TcpStream::connect(format!("{}:{}", server, port)),
                    ).await;
                    let ok = connect_result.is_ok() && connect_result.unwrap().is_ok();
                    let latency = if ok { start.elapsed().as_secs_f64() * 1000.0 } else { 0.0 };

                    // Emit progress event so frontend updates
                    let _ = app.emit("speed-test-progress", json!({
                        "current": 0,
                        "total": 0,
                        "node_id": node_id,
                        "status": if ok { "done" } else { "failed" },
                        "latency_ms": if ok { json!(latency) } else { json!(null) },
                    }));

                    // Update DB
                    let manager_arc = match self_weak.as_ref().and_then(|w| w.upgrade()) {
                        Some(arc) => arc,
                        None => break,
                    };
                    let mgr = manager_arc.lock().await;
                    if let Some(db) = &mgr.db {
                        let conn = db.lock().await;
                        let now = Utc::now().to_rfc3339();
                        let lat = if ok { Some(latency) } else { None };
                        conn.execute(
                            "UPDATE nodes SET latency_ms = ?1, last_test_at = ?2 WHERE id = ?3",
                            rusqlite::params![lat, now, node_id],
                        ).ok();
                    }
                }

                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        });

        self.auto_ping_task = Some(handle);
    }

    fn stop_auto_ping(&mut self) {
        if let Some(handle) = self.auto_ping_task.take() {
            handle.abort();
        }
    }
}

/// Inner function to refresh subscription from URL using DB directly.
/// Takes `db` directly to avoid `app.state()` in spawned tasks.
async fn refresh_subscription_inner(app: &AppHandle, url: &str) -> Result<usize, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    let decoded = if let Ok(bytes) = base64_decode(&text) {
        String::from_utf8(bytes).unwrap_or(text.clone())
    } else {
        text
    };

    let nodes: Vec<Value> = if let Ok(arr) = serde_json::from_str::<Vec<Value>>(&decoded) {
        arr
    } else {
        let mut nodes = Vec::new();
        for line in decoded.lines() {
            let line = line.trim();
            if !line.is_empty() {
                if let Ok(node) = parse_share_link(line) {
                    nodes.push(node);
                }
            }
        }
        nodes
    };

    // Use AppHandle state to access DB
    let existing_nodes = {
        let state = app.state::<crate::AppState>();
        let mgr = state.vpn_manager.lock().await;
        mgr.get_nodes().await.unwrap_or_default()
    };

    let mut import_count = 0;
    for node in &nodes {
        let server = node.get("server").and_then(|v| v.as_str()).unwrap_or("");
        let port = node.get("port").and_then(|v| v.as_i64()).unwrap_or(0);
        let name = node.get("name").and_then(|v| v.as_str()).unwrap_or("");

        let is_duplicate = existing_nodes.iter().any(|n| {
            n.get("server").and_then(|v| v.as_str()) == Some(server)
                && n.get("port").and_then(|v| v.as_i64()) == Some(port)
        });

        if !is_duplicate {
            let state = app.state::<crate::AppState>();
            let mgr = state.vpn_manager.lock().await;
            mgr.add_node(node.clone()).await.ok();
            import_count += 1;
            let _ = app.emit("subscription-imported", json!({
                "name": name,
                "server": server,
                "port": port,
            }));
        }
    }

    Ok(import_count)
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::Engine as _;
    let input = input.trim();
    let input = input
        .lines()
        .filter(|l| !l.starts_with("//") && !l.starts_with('#'))
        .collect::<Vec<_>>()
        .join("");

    if let Ok(data) = base64::engine::general_purpose::STANDARD.decode(&input) {
        return Ok(data);
    }
    if let Ok(data) = base64::engine::general_purpose::URL_SAFE.decode(&input) {
        return Ok(data);
    }
    Err("Invalid base64 input".to_string())
}

mod wait_for_proxy {
    use std::process::Child;
    use std::time::Duration;

    pub(super) async fn wait(addr: &str, max_retries: u32, delay_ms: u64, mut child: Option<&mut Child>) -> bool {
        let sock_addr: std::net::SocketAddr = match addr.parse() {
            Ok(a) => a,
            Err(_) => return false,
        };

        for i in 0..max_retries {
            // Quick process check (non-blocking) — if dead, abort immediately
            if let Some(c) = child.as_mut() {
                if c.try_wait().ok().flatten().is_some() {
                    log::warn!("wait_for_proxy: process died on attempt {}", i + 1);
                    return false;
                }
            }

            match tokio::time::timeout(Duration::from_secs(1), tokio::net::TcpStream::connect(sock_addr)).await {
                Ok(Ok(_)) => return true,
                _ => {
                    if i > 0 && i % 4 == 0 {
                        log::warn!("wait_for_proxy: retry {}/{}", i, max_retries);
                    }
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }
        false
    }
}

fn parse_share_link(line: &str) -> Result<Value, String> {
    let line = line.trim();

    if line.starts_with("vless://") {
        parse_vless(line)
    } else if line.starts_with("vmess://") {
        parse_vmess(line)
    } else if line.starts_with("trojan://") {
        parse_trojan(line)
    } else if line.starts_with("ss://") {
        parse_shadowsocks(line)
    } else if line.starts_with("hysteria2://") || line.starts_with("hy2://") {
        parse_hysteria2(line)
    } else if line.starts_with("wireguard://") || line.starts_with("wg://") {
        parse_wireguard(line)
    } else {
        Err(format!("Unknown protocol in link: {line}"))
    }
}

fn parse_vless(link: &str) -> Result<Value, String> {
    let without_prefix = link.trim_start_matches("vless://");
    let (userinfo, fragment) = without_prefix.split_once('#').unwrap_or((without_prefix, ""));
    let (credentials, rest) = userinfo.split_once('@').unwrap_or((userinfo, ""));

    let uuid = credentials;
    let (host, query) = rest.split_once('?').unwrap_or((rest, ""));
    let (server, port_str) = host.split_once(':').unwrap_or((host, "443"));
    let port: u16 = port_str.parse().unwrap_or(443);

    let params: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    let name = fragment.trim_start_matches('#');

    let config = json!({
        "uuid": uuid,
        "type": params.get("type").map(|s| s.as_str()).unwrap_or("tcp"),
        "security": params.get("security").map(|s| s.as_str()).unwrap_or("none"),
        "flow": params.get("flow").map(|s| s.as_str()).unwrap_or(""),
        "encryption": params.get("encryption").map(|s| s.as_str()).unwrap_or("none"),
        "fingerprint": params.get("fp").map(|s| s.as_str()).unwrap_or(""),
        "public_key": params.get("pbk").map(|s| s.as_str()).unwrap_or(""),
        "short_id": params.get("sid").map(|s| s.as_str()).unwrap_or(""),
        "sni": params.get("sni").map(|s| s.as_str()).unwrap_or(""),
        "network": params.get("network").map(|s| s.as_str()).unwrap_or("tcp"),
    });

    Ok(json!({
        "name": if name.is_empty() { format!("VLESS-{}", server) } else { name.to_string() },
        "protocol": "vless",
        "server": server,
        "port": port,
        "config": config,
    }))
}

fn parse_vmess(link: &str) -> Result<Value, String> {
    let b64 = link.trim_start_matches("vmess://");
    let decoded = base64_decode(b64)?;

    let vmess: Value = serde_json::from_slice(&decoded)
        .map_err(|e| format!("Failed to parse VMess JSON: {e}"))?;

    let server = vmess.get("add").and_then(|v| v.as_str()).unwrap_or("");
    let port = vmess.get("port").and_then(|v| {
        if let Some(s) = v.as_str() { s.parse().ok() } else { v.as_i64().map(|i| i as u16) }
    }).unwrap_or(443);
    let name = vmess.get("ps").and_then(|v| v.as_str()).unwrap_or(server);

    Ok(json!({
        "name": name,
        "protocol": "vmess",
        "server": server,
        "port": port,
        "config": vmess,
    }))
}

fn parse_trojan(link: &str) -> Result<Value, String> {
    let without_prefix = link.trim_start_matches("trojan://");
    let (userinfo, fragment) = without_prefix.split_once('#').unwrap_or((without_prefix, ""));
    let (password, rest) = userinfo.split_once('@').unwrap_or((userinfo, ""));
    let (host, query) = rest.split_once('?').unwrap_or((rest, ""));
    let (server, port_str) = host.split_once(':').unwrap_or((host, "443"));
    let port: u16 = port_str.parse().unwrap_or(443);

    let params: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    let name = fragment.trim_start_matches('#');

    let config = json!({
        "password": password,
        "sni": params.get("sni").map(|s| s.as_str()).unwrap_or(""),
        "allow_insecure": params.get("allowInsecure").map(|s| s == "1").unwrap_or(false),
    });

    Ok(json!({
        "name": if name.is_empty() { format!("Trojan-{}", server) } else { name.to_string() },
        "protocol": "trojan",
        "server": server,
        "port": port,
        "config": config,
    }))
}

fn parse_shadowsocks(link: &str) -> Result<Value, String> {
    let without_prefix = link.trim_start_matches("ss://");
    let (b64_part, fragment) = without_prefix.split_once('#').unwrap_or((without_prefix, ""));

    let (userinfo, host) = if let Some((user, h)) = b64_part.split_once('@') {
        (user.to_string(), h.to_string())
    } else {
        let decoded = base64_decode(b64_part)
            .and_then(|d| String::from_utf8(d).map_err(|e| e.to_string()))?;
        let (method_and_pass, h) = decoded.split_once('@').unwrap_or((&decoded, ""));
        (method_and_pass.to_string(), h.to_string())
    };

    let decoded_user = base64_decode(&userinfo)
        .and_then(|d| String::from_utf8(d).map_err(|e| e.to_string()))
        .unwrap_or_else(|_| userinfo.to_string());

    let (method, password) = decoded_user.split_once(':').unwrap_or((&decoded_user, ""));
    let (server, port_str) = host.split_once(':').unwrap_or((&host, "443"));
    let port: u16 = port_str.parse().unwrap_or(443);
    let name = fragment.trim_start_matches('#');

    let config = json!({
        "method": method,
        "password": password,
    });

    Ok(json!({
        "name": if name.is_empty() { format!("SS-{}", server) } else { name.to_string() },
        "protocol": "shadowsocks",
        "server": server,
        "port": port,
        "config": config,
    }))
}

fn parse_hysteria2(link: &str) -> Result<Value, String> {
    let without_prefix = link
        .trim_start_matches("hysteria2://")
        .trim_start_matches("hy2://");

    let (userinfo, fragment) = without_prefix.split_once('#').unwrap_or((without_prefix, ""));
    let (password, rest) = userinfo.split_once('@').unwrap_or((userinfo, ""));
    let (host, query) = rest.split_once('?').unwrap_or((rest, ""));
    let (server, port_str) = host.split_once(':').unwrap_or((host, "443"));
    let port: u16 = port_str.parse().unwrap_or(443);

    let params: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    let name = fragment.trim_start_matches('#');

    let config = json!({
        "password": password,
        "sni": params.get("sni").map(|s| s.as_str()).unwrap_or(""),
        "insecure": params.get("insecure").map(|s| s == "1").unwrap_or(false),
        "mport": params.get("mport").map(|s| s.as_str()).unwrap_or(""),
    });

    Ok(json!({
        "name": if name.is_empty() { format!("Hysteria2-{}", server) } else { name.to_string() },
        "protocol": "hysteria2",
        "server": server,
        "port": port,
        "config": config,
    }))
}

fn parse_wireguard(link: &str) -> Result<Value, String> {
    let without_prefix = link
        .trim_start_matches("wireguard://")
        .trim_start_matches("wg://");

    let (userinfo, fragment) = without_prefix.split_once('#').unwrap_or((without_prefix, ""));
    let (private_key, rest) = userinfo.split_once('@').unwrap_or((userinfo, ""));
    let (host, query) = rest.split_once('?').unwrap_or((rest, ""));
    let (server, port_str) = host.split_once(':').unwrap_or((host, "51820"));
    let port: u16 = port_str.parse().unwrap_or(51820);

    let params: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    let name = fragment.trim_start_matches('#');

    let config = json!({
        "private_key": private_key,
        "public_key": params.get("public_key").map(|s| s.as_str()).unwrap_or(""),
        "address": params.get("address").map(|s| s.as_str()).unwrap_or(""),
        "dns": params.get("dns").map(|s| s.as_str()).unwrap_or(""),
        "mtu": params.get("mtu").and_then(|s| s.parse::<u16>().ok()).unwrap_or(1420),
        "allowed_ips": params.get("allowed_ips").map(|s| s.as_str()).unwrap_or("0.0.0.0/0"),
        "persistent_keepalive": params.get("persistent_keepalive").and_then(|s| s.parse::<u16>().ok()).unwrap_or(25),
    });

    Ok(json!({
        "name": if name.is_empty() { format!("WG-{}", server) } else { name.to_string() },
        "protocol": "wireguard",
        "server": server,
        "port": port,
        "config": config,
    }))
}
