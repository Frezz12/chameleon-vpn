use log::info;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

use crate::AppState;

pub struct TrayManager {
    app: AppHandle,
    tray_handle: Option<tauri::tray::TrayIcon>,
}

impl TrayManager {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            tray_handle: None,
        }
    }

    pub fn set_tray(&mut self, handle: tauri::tray::TrayIcon) {
        self.tray_handle = Some(handle);
    }

    pub async fn update_tray_menu(&self, status: &crate::vpn_manager::VpnStatus) {
        if let Some(tray) = &self.tray_handle {
            let app = &self.app;
            let status_text = if status.connected { "Connected" } else { "Disconnected" };
            let speed_text = format!("Download: {:.1} Mbit/s | Upload: {:.1} Mbit/s",
                status.download_speed, status.upload_speed);

            // Build menu items
            let status_item = MenuItem::with_id(app, "status", status_text, true, None::<&str>)
                .unwrap_or_else(|_| panic!("Failed to create status menu item"));
            let speed_item = MenuItem::with_id(app, "speed", &speed_text, true, None::<&str>)
                .unwrap_or_else(|_| panic!("Failed to create speed menu item"));
            let separator1 = PredefinedMenuItem::separator(app).unwrap();
            let separator2 = PredefinedMenuItem::separator(app).unwrap();
            let separator3 = PredefinedMenuItem::separator(app).unwrap();

            let show_item = MenuItem::with_id(app, "show", "Show Window (Ctrl+Shift+S)", false, None::<&str>)
                .unwrap_or_else(|_| panic!("Failed to create show menu item"));
            let exit_item = MenuItem::with_id(app, "exit", "Exit", false, None::<&str>)
                .unwrap_or_else(|_| panic!("Failed to create exit menu item"));

            // Build nodes submenu dynamically
            let node_items: Vec<tauri::menu::MenuItem<tauri::Wry>> = if let Some(state) = app.try_state::<AppState>() {
                if let Ok(nodes) = state.vpn_manager.lock().await.get_nodes().await {
                    nodes.iter().filter_map(|node| {
                        let name = node.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                        let node_id = node.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let is_active = status.current_node_id.as_deref() == Some(node_id);
                        let label = if is_active {
                            format!("✓ {name}")
                        } else {
                            name.to_string()
                        };
                        MenuItem::with_id(app, &format!("node-{node_id}"), &label, false, None::<&str>).ok()
                    }).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };
            let node_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = node_items.iter().map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect();

            let nodes_submenu = Submenu::with_items(app, "Nodes", true, &node_refs)
                .unwrap_or_else(|_| panic!("Failed to create nodes submenu"));

            let menu_items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = vec![
                &status_item,
                &speed_item,
                &separator1,
                &nodes_submenu as &dyn tauri::menu::IsMenuItem<tauri::Wry>,
                &separator2,
                &show_item,
                &separator3,
                &exit_item,
            ];

            let menu = Menu::with_items(app, &menu_items)
                .unwrap_or_else(|_| panic!("Failed to create tray menu"));

            let _ = tray.set_menu(Some(menu));
        }
    }
}

pub fn create_tray(app: &AppHandle) -> Result<tauri::tray::TrayIcon, Box<dyn std::error::Error>> {
    // Create a simple icon (1x1 pixel PNG) as placeholder
    let icon = create_default_icon();

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("Chameleon")
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                id if id == "exit" => {
                    info!("Exiting via tray menu — cleaning up proxy");
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = app_clone.state::<crate::AppState>();
                        let mut mgr = state.vpn_manager.lock().await;
                        mgr.force_cleanup();
                    });
                    // Small delay to let cleanup run
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    app.exit(0);
                }
                id if id == "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                id if id.starts_with("node-") => {
                    let node_id = id.trim_start_matches("node-").to_string();
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = app_clone.state::<AppState>();
                        let mut mgr = state.vpn_manager.lock().await;
                        if mgr.get_status().ok().and_then(|s| s.get("connected").and_then(|v| v.as_bool())).unwrap_or(false) {
                            let _ = mgr.switch_node(&node_id).await;
                        } else {
                            let _ = mgr.connect(&node_id).await;
                        }
                    });
                }
                id if id == "status" => {
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = app_clone.state::<AppState>();
                        let mut mgr = state.vpn_manager.lock().await;
                        if mgr.get_status().ok().and_then(|s| s.get("connected").and_then(|v| v.as_bool())).unwrap_or(false) {
                            let _ = mgr.disconnect().await;
                        } else if let Some(node_id) = mgr.get_nodes().await.ok()
                            .and_then(|nodes| nodes.first().cloned())
                            .and_then(|n| n.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                        {
                            let _ = mgr.connect(&node_id).await;
                        }
                    });
                }
                _ => {}
            }
        })
        .build(app)?;

    info!("System tray created successfully");
    Ok(tray)
}

fn create_default_icon() -> Image<'static> {
    let mut rgba = Vec::with_capacity(32 * 32 * 4);
    for y in 0..32u32 {
        for x in 0..32u32 {
            let dx = (x as f64 - 15.5) / 15.5;
            let dy = (y as f64 - 15.5) / 15.5;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= 1.0 {
                rgba.push(0x22); rgba.push(0xBB); rgba.push(0x66); rgba.push(255);
            } else if dist <= 1.1 {
                rgba.push(0xFF); rgba.push(0xFF); rgba.push(0xFF); rgba.push(64);
            } else {
                rgba.push(0); rgba.push(0); rgba.push(0); rgba.push(0);
            }
        }
    }
    Image::new_owned(rgba, 32, 32)
}

pub fn register_shortcuts(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let app_v = app.clone();
    let _ = app.global_shortcut().on_shortcut(
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV),
        move |_app, _event, _state| {
            let app = app_v.clone();
            tauri::async_runtime::spawn(async move {
                let state = app.state::<AppState>();
                let mut mgr = state.vpn_manager.lock().await;
                if mgr.get_status().ok().and_then(|s| s.get("connected").and_then(|v| v.as_bool())).unwrap_or(false) {
                    let _ = mgr.disconnect().await;
                } else if let Ok(nodes) = mgr.get_nodes().await {
                    if let Some(first) = nodes.first() {
                        if let Some(node_id) = first.get("id").and_then(|v| v.as_str()) {
                            let _ = mgr.connect(node_id).await;
                        }
                    }
                }
            });
        },
    );

    let _ = app.global_shortcut().on_shortcut(
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyS),
        |app, _event, _state| {
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        },
    );

    let _ = app.global_shortcut().on_shortcut(
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyL),
        |app, _event, _state| {
            let _ = app.emit("toggle-log", ());
        },
    );

    info!("Global shortcuts registered");
    Ok(())
}
