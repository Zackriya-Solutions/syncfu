use crate::types::{NotificationPayload, NotificationUpdate};

/// Trait for UI-layer notification callbacks.
///
/// The HTTP server calls these methods when notifications change.
/// Desktop app implements this with Tauri panel/emit calls.
/// Headless mode uses `NoopNotifier`.
pub trait UiNotifier: Send + Sync + 'static {
    fn on_added(&self, payload: &NotificationPayload);
    fn on_updated(&self, id: &str, update: &NotificationUpdate);
    fn on_dismissed(&self, id: &str, remaining: usize);
    fn on_all_dismissed(&self, count: usize);
}

/// No-op notifier for headless server mode.
pub struct NoopNotifier;

impl UiNotifier for NoopNotifier {
    fn on_added(&self, payload: &NotificationPayload) {
        log::info!(
            "[headless] notification added: id={} title={}",
            payload.id,
            payload.title
        );
    }

    fn on_updated(&self, id: &str, _update: &NotificationUpdate) {
        log::info!("[headless] notification updated: id={id}");
    }

    fn on_dismissed(&self, id: &str, remaining: usize) {
        log::info!("[headless] notification dismissed: id={id} remaining={remaining}");
    }

    fn on_all_dismissed(&self, count: usize) {
        log::info!("[headless] all dismissed: count={count}");
    }
}
