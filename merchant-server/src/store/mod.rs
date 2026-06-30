//! Checkout persistence layer.
//!
//! `CheckoutStore` is the storage-agnostic interface used by the routes.
//! Phase 2 implements it with PostgreSQL (see `postgres.rs`); a future
//! Oracle ADB implementation can be added behind the same trait without
//! touching `routes/checkout.rs`.

mod postgres;

pub use postgres::PgCheckoutStore;

use async_trait::async_trait;
use crate::models::checkout::Checkout;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("checkout session not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[async_trait]
pub trait CheckoutStore: Send + Sync {
    /// Persists a brand-new checkout session. Callers are expected to pass
    /// a checkout with an id that does not already exist in the store.
    async fn insert(&self, checkout: Checkout) -> Result<(), StoreError>;

    /// Fetches a checkout session by id.
    async fn get(&self, id: &str) -> Result<Checkout, StoreError>;

    /// Persists an existing checkout session, overwriting its prior state.
    /// Returns `StoreError::NotFound` if no session exists with this id.
    async fn save(&self, checkout: &Checkout) -> Result<(), StoreError>;
}
