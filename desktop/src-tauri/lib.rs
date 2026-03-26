pub mod overlay;
pub mod tray;

use std::sync::Arc;

use log::{error, info};
use syncfu_core::manager::NotificationManager;
use syncfu_core::notifier::UiNotifier;
use syncfu_core::server::ServerState;
use syncfu_core::types::{NotificationPayload, NotificationUpdate, Priority, Timeout};
use syncfu_core::waiters::{WaitEvent, WaiterRegistry};
use syncfu_core::webhook::{self, WebhookPayload, WebhookResult};
use tauri::{Emitter, Manager};
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

#[cfg(debug_assertions)]
const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Debug;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

/// Tauri-backed UI notifier — shows/hides the overlay panel and emits events to the frontend.
struct TauriNotifier {
    app_handle: tauri::AppHandle,
}

impl UiNotifier for TauriNotifier {
    fn on_added(&self, payload: &NotificationPayload) {
        overlay::panel::show_panel(&self.app_handle);
        if let Err(e) = self.app_handle.emit("notification:add", payload) {
            error!("Failed to emit notification:add: {e}");
        }
    }

    fn on_updated(&self, id: &str, update: &NotificationUpdate) {
        let _ = self.app_handle.emit(
            "notification:update",
            &serde_json::json!({ "id": id, "update": update }),
        );
    }

    fn on_dismissed(&self, id: &str, remaining: usize) {
        let _ = self.app_handle.emit("notification:dismiss", id);
        if remaining == 0 {
            overlay::panel::hide_panel(&self.app_handle);
        }
    }

    fn on_all_dismissed(&self, _count: usize) {
        overlay::panel::hide_panel(&self.app_handle);
        let _ = self.app_handle.emit("notification:dismiss-all", &_count);
    }
}

#[tauri::command]
async fn notify(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    notifier: tauri::State<'_, Arc<dyn UiNotifier>>,
    payload: NotificationPayload,
) -> Result<String, String> {
    let id = manager.add(payload.clone()).await;
    notifier.on_added(&payload);
    Ok(id)
}

#[tauri::command]
async fn dismiss_notification(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    waiters: tauri::State<'_, Arc<WaiterRegistry>>,
    notifier: tauri::State<'_, Arc<dyn UiNotifier>>,
    id: String,
) -> Result<bool, String> {
    let dismissed = manager.dismiss(&id).await;
    if dismissed.is_some() {
        waiters.notify(&id, WaitEvent::Dismissed).await;
        let remaining = manager.active_count().await;
        notifier.on_dismissed(&id, remaining);
    }
    Ok(dismissed.is_some())
}

#[tauri::command]
async fn dismiss_all(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    waiters: tauri::State<'_, Arc<WaiterRegistry>>,
    notifier: tauri::State<'_, Arc<dyn UiNotifier>>,
) -> Result<usize, String> {
    waiters.notify_all(WaitEvent::Dismissed).await;
    let dismissed = manager.dismiss_all().await;
    let count = dismissed.len();
    notifier.on_all_dismissed(count);
    Ok(count)
}

#[tauri::command]
async fn update_notification(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    notifier: tauri::State<'_, Arc<dyn UiNotifier>>,
    id: String,
    update: NotificationUpdate,
) -> Result<bool, String> {
    let updated = manager.update(&id, update.clone()).await;
    if updated {
        notifier.on_updated(&id, &update);
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

#[tauri::command]
async fn action_callback(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    waiters: tauri::State<'_, Arc<WaiterRegistry>>,
    notifier: tauri::State<'_, Arc<dyn UiNotifier>>,
    notification_id: String,
    action_id: String,
) -> Result<WebhookResult, String> {
    let notification = manager
        .get(&notification_id)
        .await
        .ok_or_else(|| format!("Notification not found: {notification_id}"))?;

    let result = if let Some(ref url) = notification.callback_url {
        let payload = WebhookPayload {
            notification_id: notification_id.clone(),
            action_id: action_id.clone(),
            sender: notification.sender.clone(),
            title: notification.title.clone(),
        };
        webhook::fire_webhook(url, &payload).await
    } else {
        info!("No callback_url for notification={notification_id}, action={action_id} — skipping webhook");
        WebhookResult {
            success: true,
            status_code: None,
            error: None,
        }
    };

    waiters
        .notify(
            &notification_id,
            WaitEvent::Action {
                action_id: action_id.clone(),
            },
        )
        .await;

    let dismissed = manager.dismiss(&notification_id).await;
    if dismissed.is_some() {
        let remaining = manager.active_count().await;
        notifier.on_dismissed(&notification_id, remaining);
    }

    Ok(result)
}

#[tauri::command]
async fn test_notify(
    manager: tauri::State<'_, Arc<NotificationManager>>,
    notifier: tauri::State<'_, Arc<dyn UiNotifier>>,
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
        style: None,
        created_at: chrono::Utc::now(),
    };
    let id = manager.add(payload.clone()).await;
    notifier.on_added(&payload);
    Ok(id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };
        eprintln!("[syncfu PANIC] at {location}: {message}");
        log::error!("[PANIC] at {location}: {message}");
    }));

    let manager = NotificationManager::new();
    let waiters = WaiterRegistry::new();

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
                .max_file_size(5 * 1024 * 1024)
                .rotation_strategy(RotationStrategy::KeepAll)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(manager.clone())
        .manage(waiters.clone());

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
            action_callback,
        ])
        .setup(move |app| {
            info!("syncfu starting up");

            // Create the TauriNotifier now that we have an app handle
            let notifier: Arc<dyn UiNotifier> = Arc::new(TauriNotifier {
                app_handle: app.handle().clone(),
            });
            app.manage(notifier.clone());

            tray::menu::setup_tray(app.handle())
                .expect("failed to set up system tray");
            info!("System tray initialized");

            overlay::panel::create_panel(app.handle())
                .expect("failed to create notification panel");
            info!("Notification panel created (hidden until first notification)");

            // Start HTTP server on port 9868
            info!("Starting HTTP server on port 9868");
            let server_state = ServerState {
                manager,
                waiters,
                notifier,
            };
            tauri::async_runtime::spawn(async move {
                if let Err(e) = syncfu_core::server::start_server(server_state, 9868).await {
                    error!("HTTP server failed: {e}");
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
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
                tray::menu::open_main_window(app);
            }
        });
}
