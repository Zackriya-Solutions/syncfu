use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, Runtime,
};

use crate::notification::manager::NotificationManager;

/// Menu item identifiers used for matching in event handlers
pub const MENU_OPEN: &str = "open_syncfu";
pub const MENU_CLEAR_ALL: &str = "clear_all";
pub const MENU_PAUSE: &str = "pause";
pub const MENU_QUIT: &str = "quit";

/// Build the system tray menu items.
/// Returns a Menu that can be set on the tray icon.
pub fn build_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, tauri::Error> {
    let open = MenuItemBuilder::with_id(MENU_OPEN, "Open syncfu").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let clear_all = MenuItemBuilder::with_id(MENU_CLEAR_ALL, "Clear All").build(app)?;
    let pause = MenuItemBuilder::with_id(MENU_PAUSE, "Pause Notifications").build(app)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id(MENU_QUIT, "Quit syncfu").build(app)?;

    Menu::with_items(app, &[&open, &separator, &clear_all, &pause, &separator2, &quit])
}

/// Set up the system tray icon with menu and event handlers.
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(app)?;

    let icon = tauri::image::Image::from_bytes(include_bytes!("../../icons/32x32.png"))
        .expect("failed to load tray icon");

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("syncfu — notification overlay")
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                id if id == MENU_OPEN => {
                    handle_open(app);
                }
                id if id == MENU_CLEAR_ALL => {
                    handle_clear_all(app);
                }
                id if id == MENU_PAUSE => {
                    // TODO: implement pause/resume toggle
                }
                id if id == MENU_QUIT => {
                    handle_quit(app);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

/// Open or create the main app window.
fn handle_open(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _ = tauri::WebviewWindowBuilder::new(
            app,
            "main",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("syncfu")
        .inner_size(900.0, 640.0)
        .min_inner_size(600.0, 400.0)
        .decorations(true)
        .resizable(true)
        .center()
        .visible(true)
        .focused(true)
        .build();
    }
}

/// Clear all active notifications.
fn handle_clear_all(app: &AppHandle) {
    let manager = app.state::<Arc<NotificationManager>>();
    let manager = manager.inner().clone();
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let dismissed = manager.dismiss_all().await;
        let count = dismissed.len();
        let _ = tauri::Emitter::emit(&app_handle, "notification:dismiss-all", &count);
    });
}

/// Quit the application with a confirmation dialog.
fn handle_quit(app: &AppHandle) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
        let confirmed = app_handle
            .dialog()
            .message("syncfu is still listening for notifications. Quit anyway?")
            .title("Quit syncfu?")
            .buttons(MessageDialogButtons::OkCancelCustom("Quit".into(), "Keep Running".into()))
            .blocking_show();

        if confirmed {
            app_handle.exit(0);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_ids_are_distinct() {
        let ids = [MENU_OPEN, MENU_CLEAR_ALL, MENU_PAUSE, MENU_QUIT];
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "Menu IDs must be unique");
    }

    #[test]
    fn menu_id_values_are_snake_case() {
        let ids = [MENU_OPEN, MENU_CLEAR_ALL, MENU_PAUSE, MENU_QUIT];
        for id in ids {
            assert!(
                id.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "Menu ID '{id}' should be snake_case"
            );
        }
    }

    #[test]
    fn menu_open_id_is_correct() {
        assert_eq!(MENU_OPEN, "open_syncfu");
    }

    #[test]
    fn menu_clear_all_id_is_correct() {
        assert_eq!(MENU_CLEAR_ALL, "clear_all");
    }

    #[test]
    fn menu_pause_id_is_correct() {
        assert_eq!(MENU_PAUSE, "pause");
    }

    #[test]
    fn menu_quit_id_is_correct() {
        assert_eq!(MENU_QUIT, "quit");
    }
}
