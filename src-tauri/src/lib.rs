pub mod notification;
pub mod overlay;
pub mod server;
pub mod tray;

use std::sync::Arc;

use log::{error, info};
use notification::manager::NotificationManager;
use notification::types::{NotificationPayload, NotificationUpdate, Priority, Timeout};
use server::http::ServerState;
use tauri::{Emitter, Manager};
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

#[cfg(debug_assertions)]
const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Debug;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

#[tauri::command]
async fn notify(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    payload: NotificationPayload,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let id = manager.add(payload.clone()).await;
    overlay::panel::show_panel(&app);
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
        // Hide panel if no more active notifications
        if manager.active_count().await == 0 {
            overlay::panel::hide_panel(&app);
        }
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
    overlay::panel::hide_panel(&app);
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
    overlay::panel::show_panel(&app);
    app.emit("notification:add", &payload)
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let manager = NotificationManager::new();

    #[cfg(debug_assertions)]
    let log_targets = vec![
        Target::new(TargetKind::LogDir {
            file_name: Some("syncfu-dev".into()),
        }),
        Target::new(TargetKind::Stdout),
        Target::new(TargetKind::Webview),
    ];

    #[cfg(not(debug_assertions))]
    let log_targets = vec![
        Target::new(TargetKind::LogDir {
            file_name: Some("syncfu".into()),
        }),
        Target::new(TargetKind::Stdout),
    ];

    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets(log_targets)
                .level(LOG_LEVEL)
                .max_file_size(5 * 1024 * 1024) // 5 MB
                .rotation_strategy(RotationStrategy::KeepAll)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(manager);

    // Register tauri-nspanel plugin on macOS for NSPanel support
    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
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
            info!("syncfu starting up");

            // Set up system tray
            tray::menu::setup_tray(app.handle())
                .expect("failed to set up system tray");
            info!("System tray initialized");

            // Create notification panel — small positioned window, top-right
            overlay::panel::create_panel(app.handle())
                .expect("failed to create notification panel");
            info!("Notification panel created (hidden until first notification)");

            // Start HTTP server on port 9868
            info!("Starting HTTP server on port 9868");
            let manager = app.state::<Arc<NotificationManager>>().inner().clone();
            let server_state = ServerState {
                manager,
                app_handle: Some(app.handle().clone()),
            };
            tauri::async_runtime::spawn(async move {
                if let Err(e) = server::http::start_server(server_state, 9868).await {
                    error!("HTTP server failed: {e}");
                }
            });

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
