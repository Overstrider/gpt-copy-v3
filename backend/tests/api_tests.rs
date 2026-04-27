use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use gpt_copy_v3_backend::{AppState, OpenRouterConfig, init_db, router};
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn test_app() -> axum::Router {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    init_db(&pool).await.unwrap();
    router(AppState::new(
        pool,
        OpenRouterConfig {
            api_key: None,
            model: "openrouter/free".to_string(),
        },
        Some("mocked assistant reply".to_string()),
    ))
}

async fn json_response(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn health_returns_ok() {
    let app = test_app().await;
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
    assert_eq!(json_response(response).await["status"], "ok");
}

#[tokio::test]
async fn chat_rejects_empty_messages() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/chat")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "message": " " }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        json_response(response).await["error"]["code"],
        "validation_error"
    );
}

#[tokio::test]
async fn chat_persists_user_and_mocked_assistant_messages() {
    let app = test_app().await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/chat")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "message": "hello" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let payload = json_response(response).await;
    let conversation_id = payload["conversation_id"].as_str().unwrap();
    assert_eq!(
        payload["assistant_message"]["content"],
        "mocked assistant reply"
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/conversations/{conversation_id}/messages"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let messages = json_response(response).await;
    assert_eq!(messages.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn chat_stream_returns_token_and_done_events() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/chat/stream")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "message": "hello" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let stream = String::from_utf8(body.to_vec()).unwrap();
    assert!(stream.contains("event: token"));
    assert!(stream.contains("event: done"));
}

#[tokio::test]
async fn conversations_can_be_created_and_listed() {
    let app = test_app().await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/conversations")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "title": "Manual chat" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/conversations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let conversations = json_response(response).await;
    assert_eq!(conversations[0]["title"], "Manual chat");
}
