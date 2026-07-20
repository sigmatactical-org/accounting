//! [`PaymentsError`].

use thiserror::Error;

/// Failure talking to the payments service.
#[derive(Debug, Error)]
pub enum PaymentsError {
    /// `ACCOUNTING_PAYMENTS_BASE_URL` is not set.
    #[error("payments integration not configured")]
    NotConfigured,
    /// The payments service answered with a non-success status.
    #[error("payments request failed: {0}")]
    Request(String),
    /// The HTTP request itself failed.
    #[error("payments request error: {0}")]
    Http(#[from] reqwest::Error),
}
