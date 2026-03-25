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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,
}

/// Per-notification style overrides. All fields optional — defaults come from CSS variables.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleOverrides {
    /// Override the priority accent color (affects icon tint, progress, countdown, top border)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
    /// Card background color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_bg: Option<String>,
    /// Card border radius (e.g. "18px")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_border_radius: Option<String>,
    /// Icon stroke/fill color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_color: Option<String>,
    /// Icon box background
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_bg: Option<String>,
    /// Icon box border color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_border_color: Option<String>,
    /// Title text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_color: Option<String>,
    /// Title font size (e.g. "14px")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_font_size: Option<String>,
    /// Body text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_color: Option<String>,
    /// Body font size (e.g. "13px")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_font_size: Option<String>,
    /// Sender label color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_color: Option<String>,
    /// Timestamp color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_color: Option<String>,
    /// Primary button background
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn_bg: Option<String>,
    /// Primary button text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn_color: Option<String>,
    /// Primary button border color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn_border_color: Option<String>,
    /// Secondary button background
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn2_bg: Option<String>,
    /// Secondary button text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn2_color: Option<String>,
    /// Secondary button border color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn2_border_color: Option<String>,
    /// Danger button background
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danger_bg: Option<String>,
    /// Danger button text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danger_color: Option<String>,
    /// Danger button border color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danger_border_color: Option<String>,
    /// Progress bar fill color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_color: Option<String>,
    /// Progress bar track color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_track_color: Option<String>,
    /// Countdown bar color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub countdown_color: Option<String>,
    /// Close button background
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_bg: Option<String>,
    /// Close button icon color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_color: Option<String>,
    /// Close button border color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_border_color: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleOverrides>,
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
            icon: None,
            bg: None,
            color: None,
            border_color: None,
        };
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.style, ActionStyle::Danger);
    }

    #[test]
    fn test_action_with_custom_style_fields() {
        let json = r##"{
            "id": "deploy",
            "label": "Deploy",
            "style": "primary",
            "icon": "rocket",
            "bg": "#22c55e",
            "color": "#ffffff",
            "borderColor": "#16a34a"
        }"##;
        let action: Action = serde_json::from_str(json).unwrap();
        assert_eq!(action.icon.as_deref(), Some("rocket"));
        assert_eq!(action.bg.as_deref(), Some("#22c55e"));
        assert_eq!(action.color.as_deref(), Some("#ffffff"));
        assert_eq!(action.border_color.as_deref(), Some("#16a34a"));
    }

    #[test]
    fn test_style_overrides_empty_json() {
        let json = "{}";
        let style: StyleOverrides = serde_json::from_str(json).unwrap();
        assert!(style.accent_color.is_none());
        assert!(style.card_bg.is_none());
        assert!(style.title_color.is_none());
    }

    #[test]
    fn test_style_overrides_partial_json() {
        let json = r##"{
            "accentColor": "#ff6b6b",
            "titleColor": "#ffffff",
            "btnBg": "#22c55e"
        }"##;
        let style: StyleOverrides = serde_json::from_str(json).unwrap();
        assert_eq!(style.accent_color.as_deref(), Some("#ff6b6b"));
        assert_eq!(style.title_color.as_deref(), Some("#ffffff"));
        assert_eq!(style.btn_bg.as_deref(), Some("#22c55e"));
        assert!(style.body_color.is_none());
    }

    #[test]
    fn test_notification_payload_with_style() {
        let json = r##"{
            "sender": "deploy",
            "title": "Deploy Complete",
            "body": "v2.0 is live",
            "style": {
                "accentColor": "#22c55e",
                "cardBg": "rgba(0, 50, 0, 0.95)",
                "iconColor": "#4ade80",
                "btnBg": "#22c55e",
                "btnColor": "#ffffff"
            }
        }"##;
        let payload: NotificationPayload = serde_json::from_str(json).unwrap();
        let style = payload.style.unwrap();
        assert_eq!(style.accent_color.as_deref(), Some("#22c55e"));
        assert_eq!(style.card_bg.as_deref(), Some("rgba(0, 50, 0, 0.95)"));
        assert_eq!(style.btn_bg.as_deref(), Some("#22c55e"));
    }

    #[test]
    fn test_style_overrides_skips_none_in_serialization() {
        let style = StyleOverrides {
            accent_color: Some("#ff0000".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&style).unwrap();
        assert!(json.contains("accentColor"));
        assert!(!json.contains("cardBg"));
        assert!(!json.contains("titleColor"));
    }
}
