// Keep in sync with src-tauri/src/notification/types.rs and src-tauri/src/server/http.rs
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

impl std::str::FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "normal" => Ok(Self::Normal),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            _ => Err(format!("invalid priority: {s} (expected: low, normal, high, critical)")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionStyle {
    Primary,
    Secondary,
    Danger,
}

impl std::str::FromStr for ActionStyle {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "primary" => Ok(Self::Primary),
            "secondary" => Ok(Self::Secondary),
            "danger" => Ok(Self::Danger),
            _ => Err(format!("invalid action style: {s} (expected: primary, secondary, danger)")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStyle {
    Bar,
    Ring,
}

impl std::str::FromStr for ProgressStyle {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bar" => Ok(Self::Bar),
            "ring" => Ok(Self::Ring),
            _ => Err(format!("invalid progress style: {s} (expected: bar, ring)")),
        }
    }
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_border_radius: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_border_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_font_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_font_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn_bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn_border_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn2_bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn2_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btn2_border_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danger_bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danger_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danger_border_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_track_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub countdown_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_border_color: Option<String>,
}

// -- Request types (what the CLI sends) --

#[derive(Debug, Serialize)]
pub struct NotifyRequest {
    pub sender: String,
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub priority: Priority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Timeout>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<ProgressInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleOverrides>,
}

#[derive(Debug, Serialize)]
pub struct UpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<ProgressInfo>,
}

#[derive(Debug, Serialize)]
pub struct ActionRequest {
    pub action_id: String,
}

// -- Response types (what the CLI receives) --

#[derive(Debug, Deserialize, Serialize)]
pub struct NotifyResponse {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub active_count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DismissAllResponse {
    pub dismissed: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookResult {
    pub success: bool,
    pub status_code: Option<u16>,
    pub error: Option<String>,
}

/// Parse "id:label:style" action spec. Style defaults to "primary".
pub fn parse_action_spec(spec: &str) -> Result<Action, String> {
    let parts: Vec<&str> = spec.splitn(3, ':').collect();
    match parts.len() {
        1 => Err("action spec requires at least id:label".to_string()),
        2 => Ok(Action {
            id: parts[0].to_string(),
            label: parts[1].to_string(),
            style: ActionStyle::Primary,
            icon: None,
            bg: None,
            color: None,
            border_color: None,
        }),
        _ => {
            let style: ActionStyle = parts[2].parse()?;
            Ok(Action {
                id: parts[0].to_string(),
                label: parts[1].to_string(),
                style,
                icon: None,
                bg: None,
                color: None,
                border_color: None,
            })
        }
    }
}

/// Parse timeout string: "never", "default", or a number of seconds.
pub fn parse_timeout(s: &str) -> Timeout {
    match s {
        "never" | "default" => Timeout::Named(s.to_string()),
        _ => match s.parse::<u64>() {
            Ok(secs) => Timeout::Seconds { seconds: secs },
            Err(_) => Timeout::Named(s.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&Priority::Critical).unwrap(), "\"critical\"");
        assert_eq!(serde_json::to_string(&Priority::Low).unwrap(), "\"low\"");
    }

    #[test]
    fn test_priority_from_str() {
        assert_eq!("high".parse::<Priority>().unwrap(), Priority::High);
        assert_eq!("HIGH".parse::<Priority>().unwrap(), Priority::High);
        assert!("invalid".parse::<Priority>().is_err());
    }

    #[test]
    fn test_timeout_named_serializes_as_string() {
        let t = Timeout::Named("never".to_string());
        assert_eq!(serde_json::to_string(&t).unwrap(), "\"never\"");
    }

    #[test]
    fn test_timeout_seconds_serializes_as_object() {
        let t = Timeout::Seconds { seconds: 30 };
        assert_eq!(serde_json::to_string(&t).unwrap(), r#"{"seconds":30}"#);
    }

    #[test]
    fn test_action_serializes_camel_case() {
        let action = Action {
            id: "ok".to_string(),
            label: "OK".to_string(),
            style: ActionStyle::Primary,
            icon: None,
            bg: Some("#fff".to_string()),
            color: None,
            border_color: Some("#000".to_string()),
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"borderColor\""));
        assert!(!json.contains("\"icon\""));
    }

    #[test]
    fn test_notify_request_serializes_correctly() {
        let req = NotifyRequest {
            sender: "ci".to_string(),
            title: "Build".to_string(),
            body: "Done".to_string(),
            icon: None,
            priority: Priority::High,
            timeout: None,
            actions: vec![],
            progress: None,
            group: None,
            theme: None,
            sound: None,
            callback_url: Some("http://localhost/cb".to_string()),
            style: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"priority\":\"high\""));
        assert!(json.contains("\"callback_url\""));
        assert!(!json.contains("\"icon\""));
        assert!(!json.contains("\"actions\""));
    }

    #[test]
    fn test_health_response_deserializes() {
        let json = r#"{"status":"ok","active_count":3}"#;
        let resp: HealthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.active_count, 3);
    }

    #[test]
    fn test_notify_response_deserializes() {
        let json = r#"{"id":"abc-123"}"#;
        let resp: NotifyResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "abc-123");
    }

    #[test]
    fn test_dismiss_all_response_deserializes() {
        let json = r#"{"dismissed":5}"#;
        let resp: DismissAllResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.dismissed, 5);
    }

    #[test]
    fn test_webhook_result_deserializes() {
        let json = r#"{"success":true,"status_code":200,"error":null}"#;
        let resp: WebhookResult = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert_eq!(resp.status_code, Some(200));
    }

    #[test]
    fn test_parse_action_spec_two_parts() {
        let action = parse_action_spec("approve:Approve").unwrap();
        assert_eq!(action.id, "approve");
        assert_eq!(action.label, "Approve");
        assert_eq!(action.style, ActionStyle::Primary);
    }

    #[test]
    fn test_parse_action_spec_three_parts() {
        let action = parse_action_spec("deny:Deny:danger").unwrap();
        assert_eq!(action.id, "deny");
        assert_eq!(action.label, "Deny");
        assert_eq!(action.style, ActionStyle::Danger);
    }

    #[test]
    fn test_parse_action_spec_one_part_fails() {
        assert!(parse_action_spec("approve").is_err());
    }

    #[test]
    fn test_parse_timeout_never() {
        matches!(parse_timeout("never"), Timeout::Named(s) if s == "never");
    }

    #[test]
    fn test_parse_timeout_seconds() {
        matches!(parse_timeout("30"), Timeout::Seconds { seconds: 30 });
    }

    #[test]
    fn test_style_overrides_skips_none() {
        let style = StyleOverrides {
            accent_color: Some("#f00".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&style).unwrap();
        assert!(json.contains("accentColor"));
        assert!(!json.contains("cardBg"));
    }
}
