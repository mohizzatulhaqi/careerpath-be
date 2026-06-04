use career_path_be::{
    app,
    config::{Config, StorageBackend},
    db::pool,
    features::project::storage::{db::DatabaseStorage, local::LocalStorage, Storage},
    state::AppState,
};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let db = pool::create_pool(&config.database_url).await?;
    let port = config.server_port;

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .map_err(|e| anyhow::anyhow!("migration failed: {e}"))?;
    tracing::info!("Database migrations applied");

    let storage: Arc<dyn Storage> = match config.storage_backend {
        StorageBackend::Database => {
            tracing::info!("Storage backend: database (PostgreSQL BYTEA)");
            Arc::new(DatabaseStorage::new(db.clone()))
        }
        StorageBackend::Local => {
            tracing::info!("Storage backend: local (root={:?})", config.storage_root);
            Arc::new(
                LocalStorage::new(config.storage_root.clone())
                    .map_err(|e| anyhow::anyhow!("failed to init local storage: {e}"))?,
            )
        }
    };

    let state = Arc::new(AppState {
        db,
        config: Arc::new(config),
        storage,
    });

    // Periodic cleanup of expired refresh tokens (runs every 6 hours).
    // Deletes ALL expired tokens — used (theft-detection records no longer needed)
    // and unused (tokens that were issued but never exchanged).
    let cleanup_db = state.db.clone();
    tokio::spawn(async move {
        let interval = tokio::time::Duration::from_secs(6 * 60 * 60);
        loop {
            match sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < NOW()")
                .execute(&cleanup_db)
                .await
            {
                Ok(r) => tracing::info!("Cleaned up {} expired refresh token(s)", r.rows_affected()),
                Err(e) => tracing::warn!("Refresh token cleanup failed: {e}"),
            }
            tokio::time::sleep(interval).await;
        }
    });

    // Keep Render free tier awake by pinging /health every 10 minutes.
    tokio::spawn(async move {
        let url = format!("http://localhost:{port}/health");
        let client = reqwest::Client::new();
        // Wait for the server to be ready before the first ping.
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
        loop {
            match client.get(&url).send().await {
                Ok(r) => tracing::info!("Keepalive ping → {}", r.status()),
                Err(e) => tracing::warn!("Keepalive ping failed: {e}"),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(10 * 60)).await;
        }
    });

    let router = app::create_app(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!("Server listening on http://0.0.0.0:{port}");
    axum::serve(listener, router).await?;

    Ok(())
}
