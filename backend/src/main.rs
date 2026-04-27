use gpt_copy_v3_backend::{AppState, OpenRouterConfig, connect_database, init_db, router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://gpt-copy.db".to_string());
    let pool = connect_database(&database_url).await?;
    init_db(&pool).await?;

    let state = AppState::new(pool, OpenRouterConfig::from_env(), None);
    let app = router(state);
    let port = std::env::var("BACKEND_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!(%addr, "starting backend");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
