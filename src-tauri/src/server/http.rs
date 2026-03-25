use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::notification::manager::NotificationManager;
use crate::notification::types::{
    Action, NotificationPayload, NotificationUpdate, Priority, ProgressInfo, Timeout,
};

/// Shared state for the HTTP server.
#[derive(Clone)]
pub struct ServerState {
    pub manager: Arc<NotificationManager>,
    pub app_handle: Option<tauri::AppHandle>,
}

/// Incoming notification request — similar to NotificationPayload but with optional fields.
#[derive(Debug, Deserialize)]
pub struct NotifyRequest {
    pub sender: String,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default = "default_priority")]
    pub priority: Priority,
    #[serde(default)]
    pub timeout: Option<Timeout>,
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub progress: Option<ProgressInfo>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub sound: Option<String>,
    #[serde(default)]
    pub callback_url: Option<String>,
}

fn default_priority() -> Priority {
    Priority::Normal
}

/// Response for POST /notify
#[derive(Debug, Serialize, Deserialize)]
pub struct NotifyResponse {
    pub id: String,
}

/// Response for POST /dismiss-all
#[derive(Debug, Serialize, Deserialize)]
pub struct DismissAllResponse {
    pub dismissed: usize,
}

/// Response for GET /health
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub active_count: usize,
}

/// Incoming update request
#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub progress: Option<ProgressInfo>,
}

/// Build the axum router.
pub fn build_router(state: ServerState) -> Router {
    Router::new()
        .route("/notify", post(handle_notify))
        .route("/notify/{id}/update", post(handle_update))
        .route("/notify/{id}/dismiss", post(handle_dismiss))
        .route("/dismiss-all", post(handle_dismiss_all))
        .route("/health", get(handle_health))
        .route("/active", get(handle_active))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Start the HTTP server on the given port.
pub async fn start_server(state: ServerState, port: u16) -> Result<(), std::io::Error> {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    info!("HTTP server listening on 127.0.0.1:{port}");
    axum::serve(listener, app).await
}

async fn handle_notify(
    State(state): State<ServerState>,
    Json(req): Json<NotifyRequest>,
) -> (StatusCode, Json<NotifyResponse>) {
    let req_sender = req.sender.clone();
    let payload = NotificationPayload {
        id: uuid::Uuid::new_v4().to_string(),
        sender: req.sender,
        title: req.title,
        body: req.body,
        icon: req.icon,
        priority: req.priority,
        timeout: req.timeout.unwrap_or_default(),
        actions: req.actions,
        progress: req.progress,
        group: req.group,
        theme: req.theme,
        sound: req.sound,
        callback_url: req.callback_url,
        created_at: chrono::Utc::now(),
    };

    let id = state.manager.add(payload.clone()).await;

    // Show panel and emit to frontend
    if let Some(ref app) = state.app_handle {
        crate::overlay::panel::show_panel(app);
        debug!("Emitting notification:add for id={id}");
        match tauri::Emitter::emit(app, "notification:add", &payload) {
            Ok(()) => info!("Notification emitted: id={id} sender={}", req_sender),
            Err(e) => error!("Failed to emit notification:add: {e}"),
        }
    } else {
        warn!("No app_handle — cannot emit notification event");
    }

    (StatusCode::CREATED, Json(NotifyResponse { id }))
}

async fn handle_update(
    State(state): State<ServerState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<UpdateRequest>,
) -> StatusCode {
    debug!("Update request for id={id}");
    let update = NotificationUpdate {
        body: req.body,
        progress: req.progress,
    };

    let updated = state.manager.update(&id, update.clone()).await;

    if updated {
        if let Some(ref app) = state.app_handle {
            let _ = tauri::Emitter::emit(
                app,
                "notification:update",
                &serde_json::json!({ "id": id, "update": update }),
            );
        }
        info!("Notification updated: id={id}");
        StatusCode::OK
    } else {
        warn!("Update failed — notification not found: id={id}");
        StatusCode::NOT_FOUND
    }
}

async fn handle_dismiss(
    State(state): State<ServerState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> StatusCode {
    debug!("Dismiss request for id={id}");
    let dismissed = state.manager.dismiss(&id).await;

    if dismissed.is_some() {
        if let Some(ref app) = state.app_handle {
            let _ = tauri::Emitter::emit(app, "notification:dismiss", &id);
            // Hide panel if no more active notifications
            if state.manager.active_count().await == 0 {
                crate::overlay::panel::hide_panel(app);
            }
        }
        info!("Notification dismissed: id={id}");
        StatusCode::OK
    } else {
        warn!("Dismiss failed — notification not found: id={id}");
        StatusCode::NOT_FOUND
    }
}

async fn handle_dismiss_all(
    State(state): State<ServerState>,
) -> Json<DismissAllResponse> {
    let dismissed = state.manager.dismiss_all().await;
    let count = dismissed.len();

    if let Some(ref app) = state.app_handle {
        crate::overlay::panel::hide_panel(app);
        let _ = tauri::Emitter::emit(app, "notification:dismiss-all", &count);
    }

    info!("All notifications dismissed: count={count}");
    Json(DismissAllResponse { dismissed: count })
}

async fn handle_health(
    State(state): State<ServerState>,
) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        active_count: state.manager.active_count().await,
    })
}

async fn handle_active(
    State(state): State<ServerState>,
) -> Json<Vec<NotificationPayload>> {
    Json(state.manager.list_active().await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_state() -> ServerState {
        ServerState {
            manager: NotificationManager::new(),
            app_handle: None,
        }
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = build_router(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(health.status, "ok");
        assert_eq!(health.active_count, 0);
    }

    #[tokio::test]
    async fn test_notify_creates_notification() {
        let state = test_state();
        let app = build_router(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/notify")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "sender": "test",
                            "title": "Hello",
                            "body": "World"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let resp: NotifyResponse = serde_json::from_slice(&body).unwrap();
        assert!(!resp.id.is_empty());

        // Verify notification was stored
        assert_eq!(state.manager.active_count().await, 1);
    }

    #[tokio::test]
    async fn test_notify_with_all_fields() {
        let app = build_router(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/notify")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "sender": "ci",
                            "title": "Build Complete",
                            "body": "All tests passed",
                            "priority": "high",
                            "actions": [
                                { "id": "open", "label": "Open PR", "style": "primary" }
                            ],
                            "progress": { "value": 0.75, "label": "75%", "style": "bar" },
                            "group": "ci-builds",
                            "sound": "success"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_dismiss_existing_notification() {
        let state = test_state();
        let payload = NotificationPayload {
            id: "test-id".to_string(),
            sender: "test".to_string(),
            title: "Test".to_string(),
            body: "Body".to_string(),
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
        };
        state.manager.add(payload).await;

        let app = build_router(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/notify/test-id/dismiss")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(state.manager.active_count().await, 0);
    }

    #[tokio::test]
    async fn test_dismiss_nonexistent_returns_404() {
        let app = build_router(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/notify/nonexistent/dismiss")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_dismiss_all() {
        let state = test_state();
        for i in 0..3 {
            let payload = NotificationPayload {
                id: format!("n{i}"),
                sender: "test".to_string(),
                title: format!("Test {i}"),
                body: "Body".to_string(),
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
            };
            state.manager.add(payload).await;
        }

        let app = build_router(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/dismiss-all")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let resp: DismissAllResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp.dismissed, 3);
        assert_eq!(state.manager.active_count().await, 0);
    }

    #[tokio::test]
    async fn test_update_existing_notification() {
        let state = test_state();
        let payload = NotificationPayload {
            id: "test-id".to_string(),
            sender: "test".to_string(),
            title: "Test".to_string(),
            body: "Original".to_string(),
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
        };
        state.manager.add(payload).await;

        let app = build_router(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/notify/test-id/update")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "body": "Updated body",
                            "progress": { "value": 0.5, "style": "bar" }
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let updated = state.manager.get("test-id").await.unwrap();
        assert_eq!(updated.body, "Updated body");
        assert!(updated.progress.is_some());
    }

    #[tokio::test]
    async fn test_update_nonexistent_returns_404() {
        let app = build_router(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/notify/nonexistent/update")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "body": "Updated"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_active_returns_list() {
        let state = test_state();
        let payload = NotificationPayload {
            id: "n1".to_string(),
            sender: "test".to_string(),
            title: "Test".to_string(),
            body: "Body".to_string(),
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
        };
        state.manager.add(payload).await;

        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/active")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let notifications: Vec<NotificationPayload> = serde_json::from_slice(&body).unwrap();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].title, "Test");
    }

    #[tokio::test]
    async fn test_health_reflects_active_count() {
        let state = test_state();
        let payload = NotificationPayload {
            id: "n1".to_string(),
            sender: "test".to_string(),
            title: "Test".to_string(),
            body: "Body".to_string(),
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
        };
        state.manager.add(payload).await;

        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(health.active_count, 1);
    }

    #[tokio::test]
    async fn test_notify_default_priority_is_normal() {
        let state = test_state();
        let app = build_router(state.clone());

        app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/notify")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&serde_json::json!({
                        "sender": "test",
                        "title": "Hello",
                        "body": "World"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

        let active = state.manager.list_active().await;
        assert_eq!(active[0].priority, Priority::Normal);
    }
}
