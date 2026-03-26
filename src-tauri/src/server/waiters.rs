use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Event sent to waiting CLI clients when a notification is resolved.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "event", rename_all = "lowercase")]
pub enum WaitEvent {
    /// SSE stream is connected and listening.
    Connected,
    /// User clicked an action button.
    Action {
        action_id: String,
    },
    /// Notification was dismissed (X button, timeout, or dismiss-all).
    Dismissed,
}

/// Registry of broadcast channels for notifications that have waiting CLI clients.
///
/// Each notification ID maps to a broadcast sender. When a notification is resolved
/// (action clicked or dismissed), the event is sent to all subscribers and the
/// entry is cleaned up.
pub struct WaiterRegistry {
    waiters: RwLock<HashMap<String, broadcast::Sender<WaitEvent>>>,
}

impl WaiterRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            waiters: RwLock::new(HashMap::new()),
        })
    }

    /// Subscribe to resolution events for a notification.
    /// Creates a new broadcast channel if one doesn't exist yet.
    /// Returns a receiver that will get the next event.
    pub async fn subscribe(&self, id: &str) -> broadcast::Receiver<WaitEvent> {
        let mut waiters = self.waiters.write().await;
        let sender = waiters
            .entry(id.to_string())
            .or_insert_with(|| broadcast::channel(4).0);
        sender.subscribe()
    }

    /// Send an event to all subscribers waiting on this notification ID.
    /// Removes the entry afterward (notification is resolved).
    pub async fn notify(&self, id: &str, event: WaitEvent) {
        let mut waiters = self.waiters.write().await;
        if let Some(sender) = waiters.remove(id) {
            // Ignore send errors — means no active receivers (CLI disconnected)
            let _ = sender.send(event);
        }
    }

    /// Send a dismissed event to all waiting notifications (used by dismiss-all).
    pub async fn notify_all(&self, event: WaitEvent) {
        let mut waiters = self.waiters.write().await;
        for (_, sender) in waiters.drain() {
            let _ = sender.send(event.clone());
        }
    }

    /// Number of notifications with active waiters (for testing/debugging).
    #[cfg(test)]
    pub async fn waiter_count(&self) -> usize {
        self.waiters.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_creates_channel() {
        let registry = WaiterRegistry::new();
        let _rx = registry.subscribe("n1").await;
        assert_eq!(registry.waiter_count().await, 1);
    }

    #[tokio::test]
    async fn test_notify_sends_action_event() {
        let registry = WaiterRegistry::new();
        let mut rx = registry.subscribe("n1").await;

        registry
            .notify("n1", WaitEvent::Action { action_id: "approve".to_string() })
            .await;

        let event = rx.recv().await.unwrap();
        assert_eq!(event, WaitEvent::Action { action_id: "approve".to_string() });
    }

    #[tokio::test]
    async fn test_notify_sends_dismissed_event() {
        let registry = WaiterRegistry::new();
        let mut rx = registry.subscribe("n1").await;

        registry.notify("n1", WaitEvent::Dismissed).await;

        let event = rx.recv().await.unwrap();
        assert_eq!(event, WaitEvent::Dismissed);
    }

    #[tokio::test]
    async fn test_notify_removes_entry() {
        let registry = WaiterRegistry::new();
        let _rx = registry.subscribe("n1").await;
        assert_eq!(registry.waiter_count().await, 1);

        registry.notify("n1", WaitEvent::Dismissed).await;
        assert_eq!(registry.waiter_count().await, 0);
    }

    #[tokio::test]
    async fn test_notify_nonexistent_is_noop() {
        let registry = WaiterRegistry::new();
        // Should not panic
        registry.notify("nonexistent", WaitEvent::Dismissed).await;
    }

    #[tokio::test]
    async fn test_notify_all_sends_to_all_waiters() {
        let registry = WaiterRegistry::new();
        let mut rx1 = registry.subscribe("n1").await;
        let mut rx2 = registry.subscribe("n2").await;

        registry.notify_all(WaitEvent::Dismissed).await;

        assert_eq!(rx1.recv().await.unwrap(), WaitEvent::Dismissed);
        assert_eq!(rx2.recv().await.unwrap(), WaitEvent::Dismissed);
        assert_eq!(registry.waiter_count().await, 0);
    }

    #[tokio::test]
    async fn test_multiple_subscribers_same_notification() {
        let registry = WaiterRegistry::new();
        let mut rx1 = registry.subscribe("n1").await;
        let mut rx2 = registry.subscribe("n1").await;

        registry
            .notify("n1", WaitEvent::Action { action_id: "ok".to_string() })
            .await;

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();
        assert_eq!(e1, WaitEvent::Action { action_id: "ok".to_string() });
        assert_eq!(e2, WaitEvent::Action { action_id: "ok".to_string() });
    }

    #[tokio::test]
    async fn test_notify_with_no_receivers_cleans_up() {
        let registry = WaiterRegistry::new();
        let rx = registry.subscribe("n1").await;
        drop(rx); // Simulate CLI disconnect

        // Should not panic, just clean up
        registry.notify("n1", WaitEvent::Dismissed).await;
        assert_eq!(registry.waiter_count().await, 0);
    }

    #[tokio::test]
    async fn test_wait_event_serialization() {
        let connected = serde_json::to_string(&WaitEvent::Connected).unwrap();
        assert_eq!(connected, r#"{"event":"connected"}"#);

        let action = serde_json::to_string(&WaitEvent::Action {
            action_id: "deploy".to_string(),
        })
        .unwrap();
        assert_eq!(action, r#"{"event":"action","action_id":"deploy"}"#);

        let dismissed = serde_json::to_string(&WaitEvent::Dismissed).unwrap();
        assert_eq!(dismissed, r#"{"event":"dismissed"}"#);
    }
}
