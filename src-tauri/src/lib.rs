mod config_gen;
mod geo_db;
mod rules_engine;
mod speed_test;
mod tray_menu;
mod vpn_manager;

use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use serde_json::json;
use tauri::Manager;
use tokio::sync::Mutex;
use tray_menu::TrayManager;
use vpn_manager::VpnManager;

pub struct AppState {
    pub vpn_manager: Arc<Mutex<VpnManager>>,
    pub tray_manager: Arc<Mutex<TrayManager>>,
    pub geo_db: Arc<geo_db::GeoDatabase>,
}

#[tauri::command]
async fn vpn_connect(node_id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut mgr = state.vpn_manager.lock().await;
    mgr.connect(&node_id).await
}

#[tauri::command]
async fn vpn_disconnect(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut mgr = state.vpn_manager.lock().await;
    mgr.disconnect().await
}

#[tauri::command]
async fn vpn_status(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.get_status()
}

#[tauri::command]
async fn verify_connection(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let mut mgr = state.vpn_manager.lock().await;
    mgr.verify_connection().await
}

#[tauri::command]
async fn vpn_switch_node(node_id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut mgr = state.vpn_manager.lock().await;
    mgr.switch_node(&node_id).await
}

#[tauri::command]
async fn get_nodes(state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.get_nodes().await
}

#[tauri::command]
async fn add_node(
    node: serde_json::Value,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.add_node(node).await
}

#[tauri::command]
async fn delete_node(node_id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.delete_node(&node_id).await
}

#[tauri::command]
async fn test_node_speed(
    node_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mgr = state.vpn_manager.lock().await;
    speed_test::test_node(&node_id, &mgr).await
}

#[tauri::command]
async fn test_all_nodes_speed(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let nodes = {
        let mgr = state.vpn_manager.lock().await;
        mgr.get_nodes().await?
    };
    let results = speed_test::test_all_nodes_from_list(&app, &nodes).await?;

    let db = {
        let mgr = state.vpn_manager.lock().await;
        mgr.db.clone()
    };

    if let Some(db) = db {
        let conn = db.lock().await;
        for result in &results {
            let Some(node_id) = result.get("node_id").and_then(|v| v.as_str()) else {
                continue;
            };
            let now = Utc::now().to_rfc3339();
            let latency = result.get("latency_ms").and_then(|v| v.as_f64());
            conn.execute(
                "UPDATE nodes SET latency_ms = ?1, last_test_at = ?2 WHERE id = ?3",
                rusqlite::params![latency, now, node_id],
            )
            .ok();
        }
    }

    Ok(results)
}

#[tauri::command]
async fn import_subscription(
    url: String,
    group_name: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let mgr = state.vpn_manager.lock().await;
    let group = group_name.unwrap_or_default();
    mgr.import_subscription(&url, &group).await
}

#[tauri::command]
async fn get_subscriptions(state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.get_subscriptions().await
}

#[tauri::command]
async fn add_subscription(
    url: String,
    name: String,
    group_name: String,
    interval_mins: i64,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.add_subscription(url, name, group_name, interval_mins).await
}

#[tauri::command]
async fn delete_subscription(sub_id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.delete_subscription(&sub_id).await
}

#[tauri::command]
async fn import_subscription_url(sub_id: String, state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let mut mgr = state.vpn_manager.lock().await;
    mgr.import_subscription_url(&sub_id).await
}

#[tauri::command]
async fn get_rules(state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.get_rules().await
}

#[tauri::command]
async fn add_rule(
    rule: serde_json::Value,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.add_rule(rule).await
}

#[tauri::command]
async fn update_rule(
    rule_id: String,
    rule: serde_json::Value,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.update_rule(&rule_id, rule).await
}

#[tauri::command]
async fn delete_rule(rule_id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.delete_rule(&rule_id).await
}

#[tauri::command]
async fn reorder_rules(
    rule_ids: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.reorder_rules(rule_ids).await
}

#[tauri::command]
async fn test_rule(
    domain: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mgr = state.vpn_manager.lock().await;
    let geo_db = &state.geo_db;
    mgr.test_rule(&domain, Some(geo_db.as_ref())).await
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.get_settings().await
}

#[tauri::command]
async fn update_settings(
    settings: serde_json::Value,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.update_settings(settings).await
}

#[tauri::command]
async fn export_logs(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.export_logs(&path).await
}

async fn make_settings_export(state: &tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let mgr = state.vpn_manager.lock().await;
    let settings = mgr.get_settings().await?;
    let nodes = mgr.get_nodes().await?;
    let rules = mgr.get_rules().await?;
    let subs = mgr.get_subscriptions().await?;
    Ok(json!({
        "settings": settings,
        "nodes": nodes,
        "rules": rules,
        "subscriptions": subs,
        "version": "0.1.0"
    }))
}

#[tauri::command]
async fn export_settings(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    make_settings_export(&state).await
}

#[tauri::command]
async fn export_settings_to_file(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let data = make_settings_export(&state).await?;
    let json = serde_json::to_string_pretty(&data).map_err(|e| format!("Serialize error: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("Failed to write settings file: {e}"))
}

#[tauri::command]
async fn import_settings(data: serde_json::Value, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mgr = state.vpn_manager.lock().await;
    mgr.replace_settings_snapshot(data).await
}

#[tauri::command]
async fn test_dns_leak(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let mgr = state.vpn_manager.lock().await;
    let status = mgr.get_status()?;
    if !status.get("connected").and_then(|v| v.as_bool()).unwrap_or(false) {
        return Err("Not connected".to_string());
    }

    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::http("http://127.0.0.1:2080").map_err(|e| format!("Proxy: {e}"))?)
        .proxy(reqwest::Proxy::https("http://127.0.0.1:2080").map_err(|e| format!("Proxy: {e}"))?)
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    // Check DNS via multiple services
    let dns_servers = [
        ("https://1.1.1.1/cdn-cgi/trace", "Cloudflare"),
        ("https://dns.google/resolve?name=example.com", "Google"),
    ];

    let mut results = Vec::new();
    for (url, name) in &dns_servers {
        match client.get(*url).send().await {
            Ok(resp) => {
                let body = resp.text().await.unwrap_or_default();
                results.push(json!({"server": name, "reachable": true, "response_len": body.len()}));
            }
            Err(e) => {
                results.push(json!({"server": name, "reachable": false, "error": e.to_string()}));
            }
        }
    }

    let all_ok = results.iter().all(|r| r.get("reachable").and_then(|v| v.as_bool()).unwrap_or(false));
    Ok(json!({
        "no_leak": all_ok,
        "results": results,
        "message": if all_ok { "DNS requests go through VPN tunnel" } else { "DNS leak detected вЂ” some DNS servers bypass VPN" }
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let vpn_manager = Arc::new(Mutex::new(VpnManager::new(app.handle().clone())));
            // Inject self-reference so background tasks can access the manager
            vpn_manager.blocking_lock().self_ref = Some(Arc::downgrade(&vpn_manager));
            let tray_manager = Arc::new(Mutex::new(TrayManager::new(app.handle().clone())));

            // Initialize GeoDatabase with fallback for dev mode
            let resource_dir = app
                .path()
                .resource_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let geo_dir = if resource_dir.join("geo").join("geoip.db").exists() {
                resource_dir.join("geo")
            } else {
                manifest_dir.join("geo")
            };
            let geo_db = Arc::new(geo_db::GeoDatabase::new(&geo_dir));

            app.manage(AppState {
                vpn_manager: vpn_manager.clone(),
                tray_manager: tray_manager.clone(),
                geo_db: geo_db.clone(),
            });

            let tray_handle = tray_menu::create_tray(app.handle())?;
            let mut tm = tray_manager.blocking_lock();
            tm.set_tray(tray_handle);

            let _ = tray_menu::register_shortcuts(app.handle());

            // Initialize database synchronously before webview loads
            vpn_manager.blocking_lock().init_db();

            // Start subscription refresh background task
            let app_clone = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_clone.state::<AppState>();
                let mut mgr = state.vpn_manager.lock().await;
                mgr.start_subscription_refresh();
                mgr.start_auto_ping();
            });

            // Cleanup proxy on window close
            let app_handle = app.handle().clone();
            if let Some(window) = app.get_webview_window("main") {
                let handle = app_handle.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        let h = handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let state = h.state::<AppState>();
                            let mut mgr = state.vpn_manager.lock().await;
                            mgr.force_cleanup();
                        });
                        // Brief pause for cleanup
                        std::thread::sleep(std::time::Duration::from_millis(150));
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            vpn_connect,
            vpn_disconnect,
            vpn_status,
            verify_connection,
            vpn_switch_node,
            get_nodes,
            add_node,
            delete_node,
            test_node_speed,
            test_all_nodes_speed,
            import_subscription,
            get_subscriptions,
            add_subscription,
            delete_subscription,
            import_subscription_url,
            get_rules,
            add_rule,
            update_rule,
            delete_rule,
            reorder_rules,
            test_rule,
            get_settings,
            update_settings,
            export_logs,
            export_settings,
            export_settings_to_file,
            import_settings,
            test_dns_leak,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}




