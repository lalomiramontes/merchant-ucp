mod models;
mod routes;
mod store;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use store::{CheckoutStore, PgCheckoutStore};

/// Shared application state, cloned into every request handler.
/// `base_url` is needed to build absolute URLs in the UCP profile response.
/// `checkout_store` persists checkout sessions in PostgreSQL (Phase 2).
#[derive(Clone)]
pub struct AppState {
    pub base_url: String,
    pub checkout_store: Arc<dyn CheckoutStore>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. postgres://user:pass@localhost/merchant_db)");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run database migrations");

    let state = AppState {
        base_url: base_url.clone(),
        checkout_store: Arc::new(PgCheckoutStore::new(pool)),
    };

    let app = Router::new()
        .route("/.well-known/ucp", get(routes::well_known::well_known_ucp))
        .route(
            "/ucp/v1/checkout-sessions",
            post(routes::checkout::create_checkout),
        )
        .route(
            "/ucp/v1/checkout-sessions/{id}",
            get(routes::checkout::get_checkout).put(routes::checkout::update_checkout),
        )
        .route(
            "/ucp/v1/checkout-sessions/{id}/complete",
            post(routes::checkout::complete_checkout),
        )
        .route(
            "/ucp/v1/checkout-sessions/{id}/cancel",
            post(routes::checkout::cancel_checkout),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("merchant-server listening on {addr}, base_url={base_url}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
