use career_path_be::{
    app,
    config::Config,
    db::pool,
    features::project::storage::local::LocalStorage,
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

    let storage = Arc::new(
        LocalStorage::new(config.storage_root.clone())
            .map_err(|e| anyhow::anyhow!("failed to init storage: {e}"))?,
    );

    let state = Arc::new(AppState {
        db,
        config: Arc::new(config),
        storage,
    });

    let router = app::create_app(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!("Server listening on http://0.0.0.0:{port}");
    axum::serve(listener, router).await?;

    Ok(())
}
