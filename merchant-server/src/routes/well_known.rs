use axum::{response::Json, extract::State};
use crate::{models::profile::UcpDiscoveryDocument, AppState};

/// GET /.well-known/ucp
///
/// This is the entry point of the UCP protocol. Any agent (Hermes, ChatGPT,
/// Gemini, etc.) discovers this merchant by fetching this URL and reading
/// which services, capabilities, and payment handlers it supports.
pub async fn well_known_ucp(State(state): State<AppState>) -> Json<UcpDiscoveryDocument> {
    Json(UcpDiscoveryDocument::for_merchant(&state.base_url))
}
