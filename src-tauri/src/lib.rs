pub mod notification;
pub mod tray;

use std::sync::Arc;

use notification::manager::NotificationManager;
use notification::types::{NotificationPayload, NotificationUpdate, Priority, Timeout};
use tauri::Emitter;

#[tauri::command]
async fn notify(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    payload: NotificationPayload,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let id = manager.add(payload.clone()).await;
    app.emit("notification:add", &payload)
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
async fn dismiss_notification(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    id: String,
    app: tauri::AppHandle,
) -> Result<bool, String> {
    let dismissed = manager.dismiss(&id).await;
    if dismissed.is_some() {
        app.emit("notification:dismiss", &id)
            .map_err(|e| e.to_string())?;
    }
    Ok(dismissed.is_some())
}

#[tauri::command]
async fn dismiss_all(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    app: tauri::AppHandle,
) -> Result<usize, String> {
    let dismissed = manager.dismiss_all().await;
    let count = dismissed.len();
    app.emit("notification:dismiss-all", &count)
        .map_err(|e| e.to_string())?;
    Ok(count)
}

#[tauri::command]
async fn update_notification(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    id: String,
    update: NotificationUpdate,
    app: tauri::AppHandle,
) -> Result<bool, String> {
    let updated = manager.update(&id, update.clone()).await;
    if updated {
        app.emit("notification:update", &serde_json::json!({ "id": id, "update": update }))
            .map_err(|e| e.to_string())?;
    }
    Ok(updated)
}

#[tauri::command]
async fn get_active_notifications(
    manager: tauri::State<'_, Arc<NotificationManager>>,
) -> Result<Vec<NotificationPayload>, String> {
    Ok(manager.list_active().await)
}

#[tauri::command]
async fn health(
    manager: tauri::State<'_, Arc<NotificationManager>>,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "status": "ok",
        "activeCount": manager.active_count().await,
    }))
}

/// Send a test notification for manual testing during development.
#[tauri::command]
async fn test_notify(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let payload = NotificationPayload {
        id: uuid::Uuid::new_v4().to_string(),
        sender: "syncfu".to_string(),
        title: "Test Notification".to_string(),
        body: "syncfu is working! This is a test notification.".to_string(),
        icon: None,
        priority: Priority::Normal,
        timeout: Timeout::default(),
        actions: vec![],
        progress: None,
        group: None,
        theme: None,
        sound: None,
        callback_url: None,
        created_at: chrono::Utc::now(),
    };
    let id = manager.add(payload.clone()).await;
    app.emit("notification:add", &payload)
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let manager = NotificationManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(manager)
        .invoke_handler(tauri::generate_handler![
            notify,
            dismiss_notification,
            dismiss_all,
            update_notification,
            get_active_notifications,
            health,
            test_notify,
        ])
        .setup(|app| {
            // Set up system tray
            tray::menu::setup_tray(app.handle())
                .expect("failed to set up system tray");

            // Get primary monitor dimensions for overlay sizing
            let (width, height) = match app.primary_monitor() {
                Ok(Some(monitor)) => {
                    let size = monitor.size();
                    (size.width as f64, size.height as f64)
                }
                _ => (1920.0, 1080.0), // fallback
            };

            // Create overlay window — fullscreen, transparent, always on top
            let _overlay = tauri::WebviewWindowBuilder::new(
                app,
                "overlay",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .transparent(true)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .shadow(false)
            .focused(false)
            .resizable(false)
            .visible(true)
            .inner_size(width, height)
            .position(0.0, 0.0)
            .title("syncfu overlay")
            .build()
            .expect("failed to create overlay window");

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide main window on close instead of destroying
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building syncfu")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                // macOS dock click — re-show the main window
                tray::menu::open_main_window(app);
            }
        });
}
