use std::time::{Duration, Instant};

use chrono::Utc;
use log::{error, info};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use tokio::time::timeout;

use crate::vpn_manager::VpnManager;

/// Test a single node's speed and latency
pub async fn test_node(node_id: &str, mgr: &VpnManager) -> Result<Value, String> {
    let nodes = mgr.get_nodes().await?;
    let node = nodes
        .into_iter()
        .find(|n| n.get("id").and_then(|v| v.as_str()) == Some(node_id))
        .ok_or_else(|| format!("Node not found: {node_id}"))?;

    let server = node
        .get("server")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let port = node
        .get("port")
        .and_then(|v| v.as_u64())
        .unwrap_or(443) as u16;

    // TCP connect latency test
    let start = Instant::now();
    let connect_result = timeout(Duration::from_secs(10), async {
        tokio::net::TcpStream::connect(format!("{server}:{port}"))
            .await
            .map(|_| start.elapsed())
    })
    .await;

    let (latency_ms, success) = match connect_result {
        Ok(Ok(elapsed)) => (elapsed.as_secs_f64() * 1000.0, true),
        Ok(Err(e)) => {
            error!("TCP connect failed for {server}:{port}: {e}");
            (f64::MAX, false)
        }
        Err(_) => {
            error!("TCP connect timeout for {server}:{port}");
            (f64::MAX, false)
        }
    };

    // Download speed test (try to download a small file to measure speed)
    let download_mbps = if success {
        measure_download_speed(&format!("{server}:{port}"), port).await
    } else {
        0.0
    };

    let result = json!({
        "node_id": node_id,
        "node_name": node.get("name"),
        "server": server,
        "port": port,
        "latency_ms": if latency_ms == f64::MAX { json!(null) } else { json!(latency_ms) },
        "download_mbps": download_mbps,
        "success": success,
        "timestamp": Utc::now().to_rfc3339(),
    });

    // Save to history if DB is available
    if let Some(db) = &mgr.db {
        let conn = db.lock().await;
        let now = Utc::now().to_rfc3339();
        let latency = if latency_ms == f64::MAX { None } else { Some(latency_ms) };
        conn.execute(
            "INSERT INTO speed_history (node_id, latency_ms, download_mbps, upload_mbps, error_rate, tested_at) VALUES (?1, ?2, ?3, 0.0, ?4, ?5)",
            rusqlite::params![
                node_id,
                latency,
                download_mbps,
                if success { 0.0 } else { 100.0 },
                now,
            ],
        ).ok();

        // Update node's latency
        if let Some(lat) = latency {
            conn.execute(
                "UPDATE nodes SET latency_ms = ?1, last_test_at = ?2 WHERE id = ?3",
                rusqlite::params![lat, now, node_id],
            ).ok();
        }
    }

    info!("Speed test for {node_id}: latency={latency_ms:.1}ms, download={download_mbps:.1}Mbps");
    Ok(result)
}

/// Test all nodes in parallel
pub async fn test_all_nodes(app: &AppHandle, mgr: &VpnManager) -> Result<Vec<Value>, String> {
    let nodes = mgr.get_nodes().await?;
    test_all_nodes_from_list(app, &nodes).await
}

/// Test all nodes from a pre-fetched list (avoids locking VpnManager)
pub async fn test_all_nodes_from_list(app: &AppHandle, nodes: &[Value]) -> Result<Vec<Value>, String> {
    let total = nodes.len();
    if total == 0 {
        app.emit("speed-test-complete", json!({ "results": [] })).ok();
        return Ok(Vec::new());
    }

    // Collect owned data to avoid lifetime issues with tokio::spawn
    let node_infos: Vec<(String, String, u16)> = nodes.iter().map(|n| {
        let id = n.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let server = n.get("server").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let port = n.get("port").and_then(|v| v.as_u64()).unwrap_or(443) as u16;
        (id, server, port)
    }).collect();

    let mut handles = Vec::new();
    for (i, (node_id, server, port)) in node_infos.into_iter().enumerate() {
        let app_clone = app.clone();

        handles.push(tokio::spawn(async move {
            let _ = app_clone.emit("speed-test-progress", json!({
                "current": i + 1,
                "total": total,
                "node_id": node_id,
                "status": "testing",
            }));

            let start = Instant::now();
            let connect_result = timeout(
                Duration::from_secs(5),
                tokio::net::TcpStream::connect(format!("{server}:{port}")),
            )
            .await;
            let ok = connect_result.is_ok() && connect_result.unwrap().is_ok();
            let latency = if ok { start.elapsed().as_secs_f64() * 1000.0 } else { f64::MAX };

            let result = json!({
                "node_id": node_id,
                "latency_ms": if latency == f64::MAX { json!(null) } else { json!(latency) },
                "success": ok,
                "timestamp": Utc::now().to_rfc3339(),
            });

            let _ = app_clone.emit("speed-test-progress", json!({
                "current": i + 1,
                "total": total,
                "node_id": node_id,
                "status": if ok { "done" } else { "failed" },
                "latency_ms": if latency == f64::MAX { json!(null) } else { json!(latency) },
            }));

            result
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    app.emit("speed-test-complete", json!({ "results": results })).ok();
    Ok(results)
}

async fn measure_download_speed(addr: &str, _port: u16) -> f64 {
    // Try to download a small file to measure speed
    // Using a simple HTTP GET to a speed test file
    let url = format!("http://{addr}/speedtest/random400x400.jpg");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .ok();

    if let Some(client) = client {
        let start = Instant::now();
        match client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(bytes) = resp.bytes().await {
                    let elapsed = start.elapsed().as_secs_f64();
                    if elapsed > 0.0 {
                        let bits = bytes.len() as f64 * 8.0;
                        return bits / elapsed / 1_000_000.0;
                    }
                }
            }
            Err(_) => {}
        }
    }

    // Fallback: return a simulated speed based on latency
    // In production, this would use actual data transfer
    0.0
}

