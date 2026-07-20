//! Client for the orders service's internal-token-gated JSON API
//! (`GET /orders/{id}`), used to validate that a bill's linked `order_id`
//! refers to an existing sales order. Accounting and orders are independently
//! owned services communicating over HTTP+JSON only — bills store the order
//! id as an opaque reference, never a database foreign key.

mod orders_error;
pub use orders_error::OrdersError;

use serde::Deserialize;
use sigma_pg::clients::http;

use crate::store::StoreError;

/// Minimal view of an order row, used to map charges back to orders during
/// receipt reconcile.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderRef {
    pub id: String,
    #[serde(default)]
    pub charge_id: Option<String>,
}

fn build_order_url(base: &str, order_id: &str) -> String {
    format!("{base}orders/{order_id}")
}

/// Every order as a minimal ref (`GET /orders`).
///
/// # Errors
///
/// [`OrdersError::NotConfigured`] when `ACCOUNTING_ORDERS_BASE_URL` is unset;
/// [`OrdersError::Request`] when the orders service answers with an error.
pub async fn fetch_order_refs() -> Result<Vec<OrderRef>, OrdersError> {
    let base = crate::config::orders_base_url().ok_or(OrdersError::NotConfigured)?;
    let url = format!("{base}orders");
    let response = http::with_internal_auth(http::client().get(url))
        .send()
        .await?;
    let response = http::ensure_success(response)
        .await
        .map_err(OrdersError::Request)?;
    Ok(response.json().await?)
}

/// Whether `order_id` names an existing sales order.
async fn order_exists(base: &str, order_id: &str) -> Result<bool, OrdersError> {
    let url = build_order_url(base, order_id);
    let response = http::with_internal_auth(http::client().get(url))
        .send()
        .await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(false);
    }
    http::ensure_success(response)
        .await
        .map_err(OrdersError::Request)?;
    Ok(true)
}

/// Validate a bill's linked order id against the orders service.
///
/// Accepts unconditionally when no order id is given or when orders
/// integration is not configured (the id is then stored as an opaque
/// reference, mirroring how line-item SKU ids work without a catalog).
///
/// # Errors
///
/// [`StoreError::OrderNotFound`] when the order doesn't exist;
/// [`StoreError::Orders`] (mapped to `502`) when the orders service can't be
/// reached.
pub async fn validate_order_link(order_id: Option<&str>) -> Result<(), StoreError> {
    let Some(id) = order_id.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(());
    };
    let Some(base) = crate::config::orders_base_url() else {
        return Ok(());
    };
    match order_exists(&base, id).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(StoreError::OrderNotFound),
        Err(e) => Err(StoreError::Orders(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_order_url_joins_base_and_id() {
        assert_eq!(
            build_order_url("http://orders.internal:8085/", "order-1"),
            "http://orders.internal:8085/orders/order-1"
        );
    }

    #[tokio::test]
    async fn validate_order_link_accepts_missing_order_id() {
        assert!(validate_order_link(None).await.is_ok());
        assert!(validate_order_link(Some("  ")).await.is_ok());
    }
}
