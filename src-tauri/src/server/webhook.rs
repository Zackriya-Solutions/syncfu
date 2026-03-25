use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Payload sent to the callback URL when a user clicks an action button.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayload {
    pub notification_id: String,
    pub action_id: String,
    pub sender: String,
    pub title: String,
}

/// Result of a webhook delivery attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookResult {
    pub success: bool,
    pub status_code: Option<u16>,
    pub error: Option<String>,
}

/// Fire a webhook POST to the given URL with the action payload.
/// Times out after 5 seconds — we don't block the UI waiting for slow endpoints.
pub async fn fire_webhook(url: &str, payload: &WebhookPayload) -> WebhookResult {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to build HTTP client for webhook: {e}");
            return WebhookResult {
                success: false,
                status_code: None,
                error: Some(e.to_string()),
            };
        }
    };

    info!(
        "Firing webhook to {} for notification={} action={}",
        url, payload.notification_id, payload.action_id
    );

    match client.post(url).json(payload).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            if resp.status().is_success() {
                info!("Webhook delivered: url={url} status={status}");
                WebhookResult {
                    success: true,
                    status_code: Some(status),
                    error: None,
                }
            } else {
                warn!("Webhook returned non-success: url={url} status={status}");
                WebhookResult {
                    success: false,
                    status_code: Some(status),
                    error: Some(format!("HTTP {status}")),
                }
            }
        }
        Err(e) => {
            error!("Webhook request failed: url={url} error={e}");
            WebhookResult {
                success: false,
                status_code: None,
                error: Some(e.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_payload_serializes_camel_case() {
        let payload = WebhookPayload {
            notification_id: "abc-123".to_string(),
            action_id: "approve".to_string(),
            sender: "ci".to_string(),
            title: "Build Complete".to_string(),
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["notificationId"], "abc-123");
        assert_eq!(json["actionId"], "approve");
        assert_eq!(json["sender"], "ci");
        assert_eq!(json["title"], "Build Complete");
    }

    #[test]
    fn test_webhook_result_success() {
        let result = WebhookResult {
            success: true,
            status_code: Some(200),
            error: None,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["statusCode"], 200);
    }

    #[test]
    fn test_webhook_result_failure() {
        let result = WebhookResult {
            success: false,
            status_code: None,
            error: Some("connection refused".to_string()),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["statusCode"].is_null());
        assert_eq!(json["error"], "connection refused");
    }

    #[tokio::test]
    async fn test_fire_webhook_to_unreachable_host() {
        let payload = WebhookPayload {
            notification_id: "test-id".to_string(),
            action_id: "click".to_string(),
            sender: "test".to_string(),
            title: "Test".to_string(),
        };
        // Port 1 is almost certainly not listening
        let result = fire_webhook("http://127.0.0.1:1/callback", &payload).await;
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}
