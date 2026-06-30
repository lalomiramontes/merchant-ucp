use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;

use crate::{
    models::checkout::{Checkout, CreateCheckoutRequest, UpdateCheckoutRequest},
    store::StoreError,
    AppState,
};

fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "checkout not found" })),
    )
}

fn internal_error() -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": "internal server error" })),
    )
}

/// POST /ucp/v1/checkout-sessions
pub async fn create_checkout(
    State(state): State<AppState>,
    Json(req): Json<CreateCheckoutRequest>,
) -> impl IntoResponse {
    let checkout = Checkout::new(req);

    match state.checkout_store.insert(checkout.clone()).await {
        Ok(_) => (StatusCode::CREATED, Json(checkout)).into_response(),
        Err(_) => internal_error().into_response(),
    }
}

/// GET /ucp/v1/checkout-sessions/:id
pub async fn get_checkout(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.checkout_store.get(&id).await {
        Ok(checkout) => (StatusCode::OK, Json(checkout)).into_response(),
        Err(StoreError::NotFound) => not_found().into_response(),
        Err(_) => internal_error().into_response(),
    }
}

/// PUT /ucp/v1/checkout-sessions/:id
pub async fn update_checkout(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(update): Json<UpdateCheckoutRequest>,
) -> impl IntoResponse {
    let mut checkout = match state.checkout_store.get(&id).await {
        Ok(c) => c,
        Err(StoreError::NotFound) => return not_found().into_response(),
        Err(_) => return internal_error().into_response(),
    };

    checkout.apply_update(update);

    match state.checkout_store.save(&checkout).await {
        Ok(_) => (StatusCode::OK, Json(checkout)).into_response(),
        Err(_) => internal_error().into_response(),
    }
}

/// POST /ucp/v1/checkout-sessions/:id/complete
pub async fn complete_checkout(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut checkout = match state.checkout_store.get(&id).await {
        Ok(c) => c,
        Err(StoreError::NotFound) => return not_found().into_response(),
        Err(_) => return internal_error().into_response(),
    };

    let complete_result = checkout.complete();

    // Persist regardless of outcome: on failure, `complete()` doesn't
    // mutate status, but we still want to surface the current state.
    if state.checkout_store.save(&checkout).await.is_err() {
        return internal_error().into_response();
    }

    match complete_result {
        Ok(_) => (StatusCode::OK, Json(checkout)).into_response(),
        Err(_msg) => (StatusCode::CONFLICT, Json(checkout)).into_response(),
    }
}

/// POST /ucp/v1/checkout-sessions/:id/cancel
pub async fn cancel_checkout(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut checkout = match state.checkout_store.get(&id).await {
        Ok(c) => c,
        Err(StoreError::NotFound) => return not_found().into_response(),
        Err(_) => return internal_error().into_response(),
    };

    checkout.cancel();

    match state.checkout_store.save(&checkout).await {
        Ok(_) => (StatusCode::OK, Json(checkout)).into_response(),
        Err(_) => internal_error().into_response(),
    }
}
