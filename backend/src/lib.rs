use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, Sse},
    },
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{convert::Infallible, str::FromStr, sync::Arc, time::Duration};
use time::OffsetDateTime;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pool: SqlitePool,
    openrouter: OpenRouterConfig,
    mock_reply: Option<String>,
}

impl AppState {
    pub fn new(pool: SqlitePool, openrouter: OpenRouterConfig, mock_reply: Option<String>) -> Self {
        Self {
            pool,
            openrouter,
            mock_reply,
        }
    }
}

#[derive(Clone)]
pub struct OpenRouterConfig {
    pub api_key: Option<Arc<String>>,
    pub model: String,
}

impl OpenRouterConfig {
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("OPENROUTER_API_KEY").ok().map(Arc::new),
            model: std::env::var("OPENROUTER_MODEL")
                .unwrap_or_else(|_| "openrouter/free".to_string()),
        }
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub conversation_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub conversation_id: String,
    pub user_message: Message,
    pub assistant_message: Message,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "validation_error"),
            AppError::Provider(_) => (StatusCode::BAD_GATEWAY, "provider_error"),
            AppError::Database(_) | AppError::Http(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };
        let body = ErrorBody {
            error: ErrorDetail {
                code,
                message: self.to_string(),
            },
        };
        (status, Json(body)).into_response()
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/api/conversations",
            get(list_conversations).post(create_conversation),
        )
        .route("/api/conversations/{id}/messages", get(list_messages))
        .route("/api/chat", post(chat))
        .route("/api/chat/stream", post(chat_stream))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn connect_database(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(conversation_id) REFERENCES conversations(id)
        );
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn list_conversations(
    State(state): State<AppState>,
) -> Result<Json<Vec<Conversation>>, AppError> {
    let rows = sqlx::query_as::<_, Conversation>(
        "SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_conversation(
    State(state): State<AppState>,
    Json(input): Json<CreateConversationRequest>,
) -> Result<Json<Conversation>, AppError> {
    let conversation = insert_conversation(
        &state.pool,
        input.title.unwrap_or_else(|| "New chat".to_string()),
    )
    .await?;
    Ok(Json(conversation))
}

async fn list_messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Message>>, AppError> {
    let rows = sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, role, content, created_at FROM messages WHERE conversation_id = ? ORDER BY created_at ASC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn chat(
    State(state): State<AppState>,
    Json(input): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let response = run_chat(&state, input).await?;
    Ok(Json(response))
}

async fn chat_stream(
    State(state): State<AppState>,
    Json(input): Json<ChatRequest>,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let response = run_chat(&state, input).await?;
    let content = response.assistant_message.content.clone();
    let stream = async_stream::stream! {
        for token in content.split_whitespace() {
            yield Ok(Event::default().event("token").data(token.to_string()));
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        yield Ok(Event::default().event("done").data(response.conversation_id));
    };
    Ok(Sse::new(stream))
}

async fn run_chat(state: &AppState, input: ChatRequest) -> Result<ChatResponse, AppError> {
    let message = input.message.trim();
    if message.is_empty() {
        return Err(AppError::Validation("message is required".to_string()));
    }

    let conversation = match input.conversation_id {
        Some(id) => id,
        None => {
            insert_conversation(&state.pool, title_from_message(message))
                .await?
                .id
        }
    };
    let user_message = insert_message(&state.pool, &conversation, "user", message).await?;
    let assistant_content = assistant_reply(state, message).await?;
    let assistant_message =
        insert_message(&state.pool, &conversation, "assistant", &assistant_content).await?;
    Ok(ChatResponse {
        conversation_id: conversation,
        user_message,
        assistant_message,
    })
}

async fn assistant_reply(state: &AppState, prompt: &str) -> Result<String, AppError> {
    if let Some(reply) = &state.mock_reply {
        return Ok(reply.clone());
    }

    let api_key =
        state.openrouter.api_key.as_ref().ok_or_else(|| {
            AppError::Provider("OPENROUTER_API_KEY is not configured".to_string())
        })?;
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "model": state.openrouter.model,
        "messages": [{ "role": "user", "content": prompt }]
    });
    let value: serde_json::Value = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key.as_str())
        .json(&payload)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let content = value["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response returned by provider")
        .to_string();
    Ok(content)
}

async fn insert_conversation(
    pool: &SqlitePool,
    title: String,
) -> Result<Conversation, sqlx::Error> {
    let now = timestamp();
    let conversation = Conversation {
        id: Uuid::new_v4().to_string(),
        title,
        created_at: now.clone(),
        updated_at: now,
    };
    sqlx::query(
        "INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&conversation.id)
    .bind(&conversation.title)
    .bind(&conversation.created_at)
    .bind(&conversation.updated_at)
    .execute(pool)
    .await?;
    Ok(conversation)
}

async fn insert_message(
    pool: &SqlitePool,
    conversation_id: &str,
    role: &str,
    content: &str,
) -> Result<Message, sqlx::Error> {
    let message = Message {
        id: Uuid::new_v4().to_string(),
        conversation_id: conversation_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        created_at: timestamp(),
    };
    sqlx::query("INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&message.id)
        .bind(&message.conversation_id)
        .bind(&message.role)
        .bind(&message.content)
        .bind(&message.created_at)
        .execute(pool)
        .await?;
    sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(&message.created_at)
        .bind(conversation_id)
        .execute(pool)
        .await?;
    Ok(message)
}

fn title_from_message(message: &str) -> String {
    let title: String = message.chars().take(48).collect();
    if title.is_empty() {
        "New chat".to_string()
    } else {
        title
    }
}

fn timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}
