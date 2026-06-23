mod models;
mod routes;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

/// Shared application state, cloned into every request handler.
/// `base_url` is needed to build absolute URLs in the UCP profile response.
#[derive(Clone)]
pub struct AppState {
    pub base_url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let state = AppState { base_url: base_url.clone() };

    let app = Router::new()
        .route("/.well-known/ucp", get(routes::well_known::well_known_ucp))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("merchant-server listening on {addr}, base_url={base_url}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
