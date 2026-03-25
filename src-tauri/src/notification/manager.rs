use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use tokio::sync::RwLock;

use super::types::{NotificationPayload, NotificationUpdate};

const MAX_VISIBLE: usize = 5;

pub struct NotificationManager {
    active: RwLock<IndexMap<String, NotificationPayload>>,
    groups: RwLock<HashMap<String, Vec<String>>>,
    queued: RwLock<Vec<NotificationPayload>>,
}

impl NotificationManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            active: RwLock::new(IndexMap::new()),
            groups: RwLock::new(HashMap::new()),
            queued: RwLock::new(Vec::new()),
        })
    }

    pub async fn add(&self, payload: NotificationPayload) -> String {
        let id = payload.id.clone();

        // Track group membership
        if let Some(ref group) = payload.group {
            let mut groups = self.groups.write().await;
            groups
                .entry(group.clone())
                .or_default()
                .push(id.clone());
        }

        let mut active = self.active.write().await;

        // Remove existing with same id (replacement)
        active.shift_remove(&id);

        if active.len() >= MAX_VISIBLE {
            // Queue overflow
            let mut queued = self.queued.write().await;
            queued.push(payload);
        } else {
            active.insert(id.clone(), payload);
        }

        id
    }

    pub async fn dismiss(&self, id: &str) -> Option<NotificationPayload> {
        let mut active = self.active.write().await;
        let removed = active.shift_remove(id);

        if removed.is_some() {
            // Promote from queue if available
            let mut queued = self.queued.write().await;
            if !queued.is_empty() && active.len() < MAX_VISIBLE {
                let promoted = queued.remove(0);
                let promoted_id = promoted.id.clone();
                active.insert(promoted_id, promoted);
            }
        }

        removed
    }

    pub async fn dismiss_all(&self) -> Vec<NotificationPayload> {
        let mut active = self.active.write().await;
        let mut queued = self.queued.write().await;
        let mut dismissed: Vec<NotificationPayload> =
            active.drain(..).map(|(_, v)| v).collect();
        dismissed.extend(queued.drain(..));
        dismissed
    }

    pub async fn update(&self, id: &str, partial: NotificationUpdate) -> bool {
        let mut active = self.active.write().await;
        if let Some(notification) = active.get_mut(id) {
            if let Some(body) = partial.body {
                notification.body = body;
            }
            if let Some(progress) = partial.progress {
                notification.progress = Some(progress);
            }
            true
        } else {
            false
        }
    }

    pub async fn get(&self, id: &str) -> Option<NotificationPayload> {
        let active = self.active.read().await;
        active.get(id).cloned()
    }

    pub async fn active_count(&self) -> usize {
        self.active.read().await.len()
    }

    pub async fn list_active(&self) -> Vec<NotificationPayload> {
        self.active.read().await.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::types::{Priority, Timeout};

    fn make_notification(id: &str) -> NotificationPayload {
        NotificationPayload {
            id: id.to_string(),
            sender: "test".to_string(),
            title: format!("Test {id}"),
            body: "body".to_string(),
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
        }
    }

    #[tokio::test]
    async fn test_add_and_get() {
        let mgr = NotificationManager::new();
        let id = mgr.add(make_notification("n1")).await;
        assert_eq!(id, "n1");

        let n = mgr.get("n1").await.unwrap();
        assert_eq!(n.title, "Test n1");
    }

    #[tokio::test]
    async fn test_add_replaces_same_id() {
        let mgr = NotificationManager::new();
        let mut n = make_notification("n1");
        mgr.add(n.clone()).await;

        n.title = "Updated".to_string();
        mgr.add(n).await;

        assert_eq!(mgr.active_count().await, 1);
        let stored = mgr.get("n1").await.unwrap();
        assert_eq!(stored.title, "Updated");
    }

    #[tokio::test]
    async fn test_dismiss() {
        let mgr = NotificationManager::new();
        mgr.add(make_notification("n1")).await;

        let dismissed = mgr.dismiss("n1").await;
        assert!(dismissed.is_some());
        assert_eq!(mgr.active_count().await, 0);
    }

    #[tokio::test]
    async fn test_dismiss_unknown_returns_none() {
        let mgr = NotificationManager::new();
        let result = mgr.dismiss("unknown").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_max_visible_queues_overflow() {
        let mgr = NotificationManager::new();
        for i in 0..7 {
            mgr.add(make_notification(&format!("n{i}"))).await;
        }

        assert_eq!(mgr.active_count().await, 5);
        let queued = mgr.queued.read().await;
        assert_eq!(queued.len(), 2);
    }

    #[tokio::test]
    async fn test_dismiss_promotes_from_queue() {
        let mgr = NotificationManager::new();
        for i in 0..6 {
            mgr.add(make_notification(&format!("n{i}"))).await;
        }

        mgr.dismiss("n0").await;
        assert_eq!(mgr.active_count().await, 5);
        assert!(mgr.queued.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_dismiss_all() {
        let mgr = NotificationManager::new();
        for i in 0..7 {
            mgr.add(make_notification(&format!("n{i}"))).await;
        }

        let dismissed = mgr.dismiss_all().await;
        assert_eq!(dismissed.len(), 7);
        assert_eq!(mgr.active_count().await, 0);
    }

    #[tokio::test]
    async fn test_update_body() {
        let mgr = NotificationManager::new();
        mgr.add(make_notification("n1")).await;

        let updated = mgr
            .update(
                "n1",
                NotificationUpdate {
                    body: Some("new body".to_string()),
                    progress: None,
                },
            )
            .await;

        assert!(updated);
        let n = mgr.get("n1").await.unwrap();
        assert_eq!(n.body, "new body");
    }

    #[tokio::test]
    async fn test_update_unknown_returns_false() {
        let mgr = NotificationManager::new();
        let result = mgr
            .update(
                "unknown",
                NotificationUpdate {
                    body: Some("x".to_string()),
                    progress: None,
                },
            )
            .await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_list_active() {
        let mgr = NotificationManager::new();
        mgr.add(make_notification("n1")).await;
        mgr.add(make_notification("n2")).await;

        let active = mgr.list_active().await;
        assert_eq!(active.len(), 2);
    }

    #[tokio::test]
    async fn test_group_tracking() {
        let mgr = NotificationManager::new();
        let mut n1 = make_notification("n1");
        n1.group = Some("ci".to_string());
        let mut n2 = make_notification("n2");
        n2.group = Some("ci".to_string());

        mgr.add(n1).await;
        mgr.add(n2).await;

        let groups = mgr.groups.read().await;
        assert_eq!(groups.get("ci").unwrap().len(), 2);
    }
}
