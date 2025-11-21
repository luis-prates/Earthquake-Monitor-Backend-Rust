use axum::{Router, routing::get};
use std::net::SocketAddr;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod ingest;
mod models;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing/logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer())
        .init();

    // Load .env if present
    dotenvy::dotenv().ok();

    // Create DB pool (not used yet but ready)
    let pool = db::init_pool().await?;

    // spawn ingestion worker in background (optional)
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        ingest::run(pool_clone).await;
    });

    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/earthquakes", get(routes::list_earthquakes))
        .route("/earthquakes/{id}", get(routes::get_earthquake))
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {}", addr);
    // run our app with hyper, listening globally on port 8000
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}
