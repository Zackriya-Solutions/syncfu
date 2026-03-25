use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Import the client and types from the crate
use syncfu_cli::client::SyncfuClient;
use syncfu_cli::types::*;

#[tokio::test]
async fn test_send_notification() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/notify"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({"id": "test-uuid-123"})),
        )
        .mount(&server)
        .await;

    let client = SyncfuClient::new(&server.uri());
    let req = NotifyRequest {
        sender: "test".to_string(),
        title: "Hello".to_string(),
        body: "World".to_string(),
        icon: None,
        priority: Priority::Normal,
        timeout: None,
        actions: vec![],
        progress: None,
        group: None,
        theme: None,
        sound: None,
        callback_url: None,
        style: None,
    };

    let resp = client.send_notification(&req).await.unwrap();
    assert_eq!(resp.id, "test-uuid-123");
}

#[tokio::test]
async fn test_send_notification_with_actions() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/notify"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({"id": "action-test"})),
        )
        .mount(&server)
        .await;

    let client = SyncfuClient::new(&server.uri());
    let req = NotifyRequest {
        sender: "ci".to_string(),
        title: "PR Ready".to_string(),
        body: "Review requested".to_string(),
        icon: Some("git-pull-request".to_string()),
        priority: Priority::High,
        timeout: Some(Timeout::Named("never".to_string())),
        actions: vec![
            Action {
                id: "approve".to_string(),
                label: "Approve".to_string(),
                style: ActionStyle::Primary,
                icon: None,
                bg: Some("#22c55e".to_string()),
                color: Some("#fff".to_string()),
                border_color: None,
            },
            Action {
                id: "deny".to_string(),
                label: "Deny".to_string(),
                style: ActionStyle::Danger,
                icon: None,
                bg: None,
                color: None,
                border_color: None,
            },
        ],
        progress: None,
        group: None,
        theme: None,
        sound: None,
        callback_url: Some("http://localhost:9870/cb".to_string()),
        style: Some(StyleOverrides {
            accent_color: Some("#a855f7".to_string()),
            ..Default::default()
        }),
    };

    let resp = client.send_notification(&req).await.unwrap();
    assert_eq!(resp.id, "action-test");
}

#[tokio::test]
async fn test_health() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"status": "ok", "active_count": 3}),
        ))
        .mount(&server)
        .await;

    let client = SyncfuClient::new(&server.uri());
    let resp = client.health().await.unwrap();
    assert_eq!(resp.status, "ok");
    assert_eq!(resp.active_count, 3);
}

#[tokio::test]
async fn test_list_active() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/active"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "n1", "sender": "ci", "title": "Build", "body": "Done", "priority": "normal", "timeout": "default", "actions": [], "createdAt": "2026-01-01T00:00:00Z"},
            {"id": "n2", "sender": "monitor", "title": "Alert", "body": "CPU high", "priority": "high", "timeout": "never", "actions": [], "createdAt": "2026-01-01T00:00:00Z"},
        ])))
        .mount(&server)
        .await;

    let client = SyncfuClient::new(&server.uri());
    let list = client.list_active().await.unwrap();
    assert_eq!(list.len(), 2);
}

#[tokio::test]
async fn test_dismiss() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/notify/test-id/dismiss"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = SyncfuClient::new(&server.uri());
    client.dismiss("test-id").await.unwrap();
}

#[tokio::test]
async fn test_dismiss_nonexistent_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/notify/nonexistent/dismiss"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = SyncfuClient::new(&server.uri());
    let result = client.dismiss("nonexistent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_dismiss_all() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/dismiss-all"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"dismissed": 5})),
        )
        .mount(&server)
        .await;

    let client = SyncfuClient::new(&server.uri());
    let resp = client.dismiss_all().await.unwrap();
    assert_eq!(resp.dismissed, 5);
}

#[tokio::test]
async fn test_update_notification() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/notify/up-id/update"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = SyncfuClient::new(&server.uri());
    let req = UpdateRequest {
        body: Some("Updated".to_string()),
        progress: Some(ProgressInfo {
            value: 0.75,
            label: Some("75%".to_string()),
            style: ProgressStyle::Bar,
        }),
    };
    client.update_notification("up-id", &req).await.unwrap();
}

#[tokio::test]
async fn test_trigger_action() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/notify/act-id/action"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({"success": true, "status_code": 200, "error": null}),
        ))
        .mount(&server)
        .await;

    let client = SyncfuClient::new(&server.uri());
    let resp = client.trigger_action("act-id", "approve").await.unwrap();
    assert!(resp.success);
    assert_eq!(resp.status_code, Some(200));
}

#[tokio::test]
async fn test_connection_error_message() {
    let client = SyncfuClient::new("http://127.0.0.1:1");
    let result = client.health().await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("cannot connect to syncfu"));
}
