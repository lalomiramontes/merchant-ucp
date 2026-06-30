use async_trait::async_trait;
use sqlx::PgPool;

use crate::models::checkout::{
    Buyer, Checkout, CheckoutMessage, CheckoutStatus, LineItem,
};
use super::{CheckoutStore, StoreError};

#[derive(Clone)]
pub struct PgCheckoutStore {
    pool: PgPool,
}

impl PgCheckoutStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Row shape as returned by SQLx for a `checkouts` SELECT.
/// Kept separate from `Checkout` because the DB representation
/// (status as TEXT, nested structs as JSONB) differs from the
/// in-memory/wire representation.
struct CheckoutRow {
    id: String,
    status: String,
    line_items: serde_json::Value,
    buyer: serde_json::Value,
    total: i64,
    currency: String,
    messages: serde_json::Value,
    continue_url: Option<String>,
    payment_handler_id: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl CheckoutRow {
    fn into_checkout(self) -> Result<Checkout, StoreError> {
        let status: CheckoutStatus = serde_json::from_value(serde_json::Value::String(self.status))?;
        let line_items: Vec<LineItem> = serde_json::from_value(self.line_items)?;
        let buyer: Buyer = serde_json::from_value(self.buyer)?;
        let messages: Vec<CheckoutMessage> = serde_json::from_value(self.messages)?;

        Ok(Checkout {
            id: self.id,
            status,
            line_items,
            buyer,
            total: self.total as u64,
            currency: self.currency,
            messages,
            continue_url: self.continue_url,
            payment_handler_id: self.payment_handler_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// Serializes `CheckoutStatus` to the plain string stored in the DB
/// (e.g. `ReadyForComplete` -> "ready_for_complete"), reusing the same
/// snake_case mapping serde already applies for the JSON API.
fn status_as_text(status: &CheckoutStatus) -> Result<String, StoreError> {
    let value = serde_json::to_value(status)?;
    Ok(value.as_str().expect("CheckoutStatus serializes to a string").to_string())
}

#[async_trait]
impl CheckoutStore for PgCheckoutStore {
    async fn insert(&self, checkout: Checkout) -> Result<(), StoreError> {
        let status = status_as_text(&checkout.status)?;
        let line_items = serde_json::to_value(&checkout.line_items)?;
        let buyer = serde_json::to_value(&checkout.buyer)?;
        let messages = serde_json::to_value(&checkout.messages)?;

        sqlx::query!(
            r#"
            INSERT INTO checkouts (
                id, status, line_items, buyer, total, currency,
                messages, continue_url, payment_handler_id,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            checkout.id,
            status,
            line_items,
            buyer,
            checkout.total as i64,
            checkout.currency,
            messages,
            checkout.continue_url,
            checkout.payment_handler_id,
            checkout.created_at,
            checkout.updated_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Checkout, StoreError> {
        let row = sqlx::query_as!(
            CheckoutRow,
            r#"
            SELECT
                id, status, line_items, buyer, total, currency,
                messages, continue_url, payment_handler_id,
                created_at, updated_at
            FROM checkouts WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StoreError::NotFound)?;

        row.into_checkout()
    }

    async fn save(&self, checkout: &Checkout) -> Result<(), StoreError> {
        let status = status_as_text(&checkout.status)?;
        let line_items = serde_json::to_value(&checkout.line_items)?;
        let buyer = serde_json::to_value(&checkout.buyer)?;
        let messages = serde_json::to_value(&checkout.messages)?;

        let result = sqlx::query!(
            r#"
            UPDATE checkouts SET
                status = $2, line_items = $3, buyer = $4, total = $5,
                currency = $6, messages = $7, continue_url = $8,
                payment_handler_id = $9, updated_at = $10
            WHERE id = $1
            "#,
            checkout.id,
            status,
            line_items,
            buyer,
            checkout.total as i64,
            checkout.currency,
            messages,
            checkout.continue_url,
            checkout.payment_handler_id,
            checkout.updated_at,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }

        Ok(())
    }
}
