use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionStyle {
    Primary,
    Secondary,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStyle {
    Bar,
    Ring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub id: String,
    pub label: String,
    pub style: ActionStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressInfo {
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub style: ProgressStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Timeout {
    Named(String),
    Seconds { seconds: u64 },
}

impl Default for Timeout {
    fn default() -> Self {
        Self::Named("default".to_string())
    }
}

impl Timeout {
    pub fn duration_secs(&self) -> Option<u64> {
        match self {
            Self::Named(s) if s == "never" => None,
            Self::Named(_) => Some(8),
            Self::Seconds { seconds } => Some(*seconds),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPayload {
    #[serde(default = "generate_id")]
    pub id: String,
    pub sender: String,
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub timeout: Timeout,
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<ProgressInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "callbackUrl")]
    pub callback_url: Option<String>,
    #[serde(default = "now")]
    pub created_at: DateTime<Utc>,
}

fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn now() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<ProgressInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub sender: String,
    pub title: String,
    pub body: String,
    pub priority: Priority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions_json: Option<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismissed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_taken: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_result: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_default_is_normal() {
        assert_eq!(Priority::default(), Priority::Normal);
    }

    #[test]
    fn test_priority_serializes_lowercase() {
        let json = serde_json::to_string(&Priority::Critical).unwrap();
        assert_eq!(json, "\"critical\"");
    }

    #[test]
    fn test_priority_deserializes_lowercase() {
        let p: Priority = serde_json::from_str("\"high\"").unwrap();
        assert_eq!(p, Priority::High);
    }

    #[test]
    fn test_timeout_default_is_8_seconds() {
        let t = Timeout::default();
        assert_eq!(t.duration_secs(), Some(8));
    }

    #[test]
    fn test_timeout_never_is_none() {
        let t = Timeout::Named("never".to_string());
        assert_eq!(t.duration_secs(), None);
    }

    #[test]
    fn test_timeout_custom_seconds() {
        let t = Timeout::Seconds { seconds: 30 };
        assert_eq!(t.duration_secs(), Some(30));
    }

    #[test]
    fn test_notification_payload_minimal_json() {
        let json = r#"{
            "sender": "ci",
            "title": "Build passed",
            "body": "All tests green"
        }"#;

        let payload: NotificationPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.sender, "ci");
        assert_eq!(payload.title, "Build passed");
        assert_eq!(payload.priority, Priority::Normal);
        assert!(payload.actions.is_empty());
        assert!(!payload.id.is_empty());
    }

    #[test]
    fn test_notification_payload_full_json() {
        let json = r#"{
            "sender": "ci-pipeline",
            "title": "Build Complete",
            "body": "**main** built in 3m 42s",
            "priority": "high",
            "timeout": { "seconds": 15 },
            "actions": [
                { "id": "open_pr", "label": "Open PR", "style": "primary" },
                { "id": "dismiss", "label": "Dismiss", "style": "secondary" }
            ],
            "progress": { "value": 0.75, "label": "3 of 4", "style": "bar" },
            "group": "ci-builds",
            "sound": "success",
            "callbackUrl": "http://localhost:8080/callback"
        }"#;

        let payload: NotificationPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.sender, "ci-pipeline");
        assert_eq!(payload.priority, Priority::High);
        assert_eq!(payload.actions.len(), 2);
        assert_eq!(payload.actions[0].id, "open_pr");
        assert_eq!(payload.progress.as_ref().unwrap().value, 0.75);
        assert_eq!(payload.group.as_deref(), Some("ci-builds"));
        assert_eq!(payload.callback_url.as_deref(), Some("http://localhost:8080/callback"));
    }

    #[test]
    fn test_notification_update_partial() {
        let json = r#"{ "body": "Updated body" }"#;
        let update: NotificationUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.body.as_deref(), Some("Updated body"));
        assert!(update.progress.is_none());
    }

    #[test]
    fn test_action_style_roundtrip() {
        let action = Action {
            id: "test".to_string(),
            label: "Test".to_string(),
            style: ActionStyle::Danger,
        };
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.style, ActionStyle::Danger);
    }
}
