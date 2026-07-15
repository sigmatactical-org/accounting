//! [`OrdersError`].

use thiserror::Error;

/// Failure talking to the orders service.
#[derive(Debug, Error)]
pub enum OrdersError {
    /// `ACCOUNTING_ORDERS_BASE_URL` is not set.
    #[error("orders integration not configured")]
    NotConfigured,
    /// The orders service answered with a non-success status.
    #[error("orders request failed: {0}")]
    Request(String),
    /// The HTTP request itself failed.
    #[error("orders request error: {0}")]
    Http(#[from] reqwest::Error),
}
