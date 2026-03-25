//! Notification panel window — a small positioned overlay instead of fullscreen.
//!
//! Platform strategy:
//! - macOS: NSPanel via tauri-nspanel (non-activating, joins all Spaces, proper z-order)
//! - Windows: WebviewWindow with always_on_top + focused(false) (Meetily pattern)
//! - Linux: Deferred (basic WebviewWindow for now)

use log::{error, info};
use tauri::{AppHandle, Manager};

#[cfg(target_os = "macos")]
use tauri_nspanel::ManagerExt;

// NSPanel type for macOS notification overlay:
// - can_become_key_window: false (never steals keyboard focus)
// - can_become_main_window: false (never becomes the main window)
// - is_floating_panel: true (floats above regular windows)
#[cfg(target_os = "macos")]
tauri_nspanel::tauri_panel! {
    panel!(NotificationPanel {
        config: {
            can_become_key_window: false,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })
}

/// Panel dimensions in logical pixels.
/// Width matches the notification card max-width.
/// Height starts minimal — frontend resizes dynamically to fit content.
pub const PANEL_WIDTH: f64 = 400.0;
pub const PANEL_INITIAL_HEIGHT: f64 = 10.0;

/// Margin from the top-right corner of the work area.
pub const MARGIN_TOP: f64 = 12.0;
pub const MARGIN_RIGHT: f64 = 12.0;

/// Position for the notification panel (logical pixels).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelPosition {
    pub x: f64,
    pub y: f64,
}

/// Monitor dimensions needed for position calculation.
#[derive(Debug, Clone, Copy)]
pub struct MonitorInfo {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

/// Calculate the top-right position for the notification panel.
///
/// Position is in logical (not physical) pixels, accounting for scale factor.
/// The panel sits at `(right_edge - panel_width - margin, top + margin)`.
pub fn calculate_panel_position(monitor: MonitorInfo) -> PanelPosition {
    let logical_x = monitor.x / monitor.scale_factor;
    let logical_y = monitor.y / monitor.scale_factor;
    let logical_width = monitor.width / monitor.scale_factor;

    PanelPosition {
        x: logical_x + logical_width - PANEL_WIDTH - MARGIN_RIGHT,
        y: logical_y + MARGIN_TOP,
    }
}

/// Create the notification panel window.
///
/// On macOS, creates an NSPanel for proper floating panel behavior.
/// On other platforms, creates a standard WebviewWindow with always-on-top.
/// The panel starts hidden and should be shown when the first notification arrives.
pub fn create_panel(app: &AppHandle) -> Result<(), String> {
    let position = match get_monitor_info(app) {
        Some(monitor) => calculate_panel_position(monitor),
        None => {
            info!("No monitor info — using default panel position");
            PanelPosition { x: 1508.0, y: 12.0 }
        }
    };

    info!(
        "Creating notification panel at ({}, {}), size {}x{}",
        position.x, position.y, PANEL_WIDTH, PANEL_INITIAL_HEIGHT
    );

    #[cfg(target_os = "macos")]
    create_macos_panel(app, position)?;

    #[cfg(not(target_os = "macos"))]
    create_standard_panel(app, position)?;

    Ok(())
}

/// macOS: Create an NSPanel via tauri-nspanel.
///
/// NSPanel provides:
/// - Non-activating (doesn't steal focus from the frontmost app)
/// - Joins all Spaces (visible on every macOS desktop)
/// - Proper z-order (above windows, below system UI)
/// - Full-screen auxiliary (visible over fullscreen apps)
#[cfg(target_os = "macos")]
fn create_macos_panel(
    app: &AppHandle,
    position: PanelPosition,
) -> Result<(), String> {
    use tauri_nspanel::PanelBuilder;

    let _panel = PanelBuilder::<_, NotificationPanel>::new(app, "overlay")
        .url(tauri::WebviewUrl::App("index.html".into()))
        .level(tauri_nspanel::PanelLevel::Status)
        .no_activate(true)
        .floating(true)
        .collection_behavior(
            tauri_nspanel::CollectionBehavior::new()
                .can_join_all_spaces()
                .full_screen_auxiliary(),
        )
        .has_shadow(true)
        .transparent(true)
        .size(tauri::Size::Logical(tauri::LogicalSize::new(PANEL_WIDTH, PANEL_INITIAL_HEIGHT)))
        .position(tauri::Position::Logical(tauri::LogicalPosition::new(position.x, position.y)))
        .with_window(|builder| {
            builder
                .transparent(true)
                .background_color(tauri::window::Color(0, 0, 0, 0))
                .decorations(false)
                .skip_taskbar(true)
                .resizable(false)
                .visible(false)
                .title("syncfu overlay")
        })
        .build()
        .map_err(|e| format!("Failed to create macOS panel: {e}"))?;

    info!("macOS NSPanel created (non-activating, joins all Spaces)");
    Ok(())
}

/// Windows/Linux: Create a standard WebviewWindow with always-on-top.
///
/// Uses the Meetily pattern:
/// - always_on_top(true) for z-order
/// - focused(false) to avoid focus theft
/// - transparent + no decorations for floating appearance
#[cfg(not(target_os = "macos"))]
fn create_standard_panel(
    app: &AppHandle,
    position: PanelPosition,
) -> Result<(), String> {
    let _window = tauri::WebviewWindowBuilder::new(
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
    .visible(false)
    .inner_size(PANEL_WIDTH, PANEL_INITIAL_HEIGHT)
    .position(position.x, position.y)
    .title("syncfu overlay")
    .build()
    .map_err(|e| format!("Failed to create panel: {e}"))?;

    info!("Standard panel created (always-on-top, non-focusing)");
    Ok(())
}

/// Show the panel window (e.g. when a notification arrives).
///
/// IMPORTANT: On macOS, NSPanel operations MUST run on the main thread.
/// This function dispatches to the main thread to avoid crashes.
pub fn show_panel(app: &AppHandle) {
    let handle = app.clone();
    let inner = app.clone();
    let _ = handle.run_on_main_thread(move || {
        #[cfg(target_os = "macos")]
        {
            if let Ok(panel) = inner.get_webview_panel("overlay") {
                panel.show();
                return;
            }
        }

        if let Some(window) = inner.get_webview_window("overlay") {
            let _ = window.show();
        }
    });
}

/// Hide the panel window (e.g. when all notifications are dismissed).
///
/// IMPORTANT: On macOS, NSPanel operations MUST run on the main thread.
/// This function dispatches to the main thread to avoid crashes.
pub fn hide_panel(app: &AppHandle) {
    let handle = app.clone();
    let inner = app.clone();
    let _ = handle.run_on_main_thread(move || {
        #[cfg(target_os = "macos")]
        {
            if let Ok(panel) = inner.get_webview_panel("overlay") {
                panel.hide();
                return;
            }
        }

        if let Some(window) = inner.get_webview_window("overlay") {
            let _ = window.hide();
        }
    });
}

/// Extract monitor info from the primary monitor.
fn get_monitor_info(app: &AppHandle) -> Option<MonitorInfo> {
    match app.primary_monitor() {
        Ok(Some(monitor)) => {
            let size = monitor.size();
            let position = monitor.position();
            let scale = monitor.scale_factor();
            Some(MonitorInfo {
                x: position.x as f64,
                y: position.y as f64,
                width: size.width as f64,
                height: size.height as f64,
                scale_factor: scale,
            })
        }
        Ok(None) => {
            error!("No primary monitor detected");
            None
        }
        Err(e) => {
            error!("Failed to get primary monitor: {e}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_position_standard_1080p() {
        let monitor = MonitorInfo {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
            scale_factor: 1.0,
        };

        let pos = calculate_panel_position(monitor);

        // Right edge: 1920 - 400 - 12 = 1508
        assert_eq!(pos.x, 1508.0);
        assert_eq!(pos.y, MARGIN_TOP);
    }

    #[test]
    fn test_panel_position_retina_display() {
        // MacBook Pro 14" — physical 3024×1964, scale 2.0
        let monitor = MonitorInfo {
            x: 0.0,
            y: 0.0,
            width: 3024.0,
            height: 1964.0,
            scale_factor: 2.0,
        };

        let pos = calculate_panel_position(monitor);

        // Logical width: 3024 / 2 = 1512
        // x: 1512 - 400 - 12 = 1100
        assert_eq!(pos.x, 1100.0);
        assert_eq!(pos.y, MARGIN_TOP);
    }

    #[test]
    fn test_panel_position_4k_150_percent_scale() {
        let monitor = MonitorInfo {
            x: 0.0,
            y: 0.0,
            width: 3840.0,
            height: 2160.0,
            scale_factor: 1.5,
        };

        let pos = calculate_panel_position(monitor);

        // Logical width: 3840 / 1.5 = 2560
        // x: 2560 - 400 - 12 = 2148
        assert_eq!(pos.x, 2148.0);
        assert_eq!(pos.y, MARGIN_TOP);
    }

    #[test]
    fn test_panel_position_secondary_monitor_offset() {
        // Secondary monitor at physical position (1920, 0)
        let monitor = MonitorInfo {
            x: 1920.0,
            y: 0.0,
            width: 2560.0,
            height: 1440.0,
            scale_factor: 1.0,
        };

        let pos = calculate_panel_position(monitor);

        // x: 1920 + 2560 - 400 - 12 = 4068
        assert_eq!(pos.x, 4068.0);
        assert_eq!(pos.y, MARGIN_TOP);
    }

    #[test]
    fn test_panel_position_secondary_monitor_retina() {
        // Secondary Retina at physical (3024, 0), 2560x1440 @ 2x
        let monitor = MonitorInfo {
            x: 3024.0,
            y: 0.0,
            width: 5120.0,
            height: 2880.0,
            scale_factor: 2.0,
        };

        let pos = calculate_panel_position(monitor);

        // Logical x: 3024 / 2 = 1512
        // Logical width: 5120 / 2 = 2560
        // x: 1512 + 2560 - 400 - 12 = 3660
        assert_eq!(pos.x, 3660.0);
        assert_eq!(pos.y, MARGIN_TOP);
    }

    #[test]
    fn test_panel_dimensions_are_reasonable() {
        assert!(PANEL_WIDTH > 300.0, "Panel too narrow for notifications");
        assert!(PANEL_WIDTH < 500.0, "Panel too wide");
        // Initial height is minimal — frontend resizes dynamically
        assert!(PANEL_INITIAL_HEIGHT <= 20.0, "Initial height should be tiny");
    }

    #[test]
    fn test_panel_fits_on_small_screen() {
        // Minimum supported: 1366x768 laptop
        let monitor = MonitorInfo {
            x: 0.0,
            y: 0.0,
            width: 1366.0,
            height: 768.0,
            scale_factor: 1.0,
        };

        let pos = calculate_panel_position(monitor);

        assert!(pos.x >= 0.0, "Panel x must be on screen");
        assert!(pos.y >= 0.0, "Panel y must be on screen");
        assert!(
            pos.x + PANEL_WIDTH <= 1366.0,
            "Panel must fit horizontally"
        );
    }
}
