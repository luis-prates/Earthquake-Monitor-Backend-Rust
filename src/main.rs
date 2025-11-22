use axum::{Router, routing::get};
use reqwest::StatusCode;
use std::net::SocketAddr;
use tracing_appender::rolling;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use earthquake_monitor_backend_rust as app;

use app::{api_docs, db, ingest, metrics, routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing/logging
    let file_appender = rolling::daily("/var/log/earthquake", "server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Console and File layer
    let console_layer = fmt::layer().with_writer(std::io::stdout);
    let file_layer = fmt::layer().with_writer(non_blocking);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(console_layer)
        .with(file_layer)
        .init();

    // Load .env if present
    dotenvy::dotenv().ok();

    // Create DB pool (not used yet but ready)
    let pool = db::init_pool().await?;

    // spawn ingestion worker in background (optional)
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let _ = ingest::run(pool_clone).await;
    });

    let app_routes = Router::new()
        .route("/health", get(routes::health))
        .route("/earthquakes", get(routes::list_earthquakes))
        .route("/earthquakes/{id}", get(routes::get_earthquake))
        .route(
            "/metrics",
            get(|| async { (StatusCode::OK, metrics::gather_metrics()) }),
        )
        .with_state(pool);

    // Mount Swagger UI at /docs, pointing to our openapi.json
    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", api_docs::ApiDoc::openapi()))
        .merge(app_routes);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {}", addr);
    // run our app with hyper, listening globally on port 8000
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}
