//! [`StoreError`].

use thiserror::Error;
use warp::http::StatusCode;

/// Accounting's store-layer error.
///
/// This stays local rather than adopting [`sigma_pg::api::StoreError`]: the
/// shared enum only distinguishes not-found / invalid-input / database, which
/// would collapse this API's status codes — a duplicate integration name has
/// to stay `409`, and an unreachable orders service has to stay `502`. The
/// shared `ErrorBody`, `json_error`, and `internal_auth` are used as-is.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("bill not found")]
    BillNotFound,
    #[error("integration not found")]
    IntegrationNotFound,
    #[error("vendor is required")]
    VendorRequired,
    #[error("bill must have at least one line item")]
    BillNeedsLineItems,
    #[error("scanned bill requires scan_uri")]
    ScanUriRequired,
    #[error("line item quantity must be at least 1")]
    InvalidQuantity,
    #[error("expense not found")]
    ExpenseNotFound,
    #[error("expense description is required")]
    ExpenseDescriptionRequired,
    #[error("expense amount must be at least 1 cent")]
    InvalidAmount,
    #[error("linked bill not found")]
    LinkedBillNotFound,
    #[error("receipt not found")]
    ReceiptNotFound,
    #[error("receipt charge_id is required")]
    ReceiptChargeRequired,
    #[error("receipt user_id is required")]
    ReceiptUserRequired,
    #[error("linked order not found")]
    OrderNotFound,
    #[error("orders service error: {0}")]
    Orders(String),
    #[error("payments service error: {0}")]
    Payments(String),
    #[error("integration name is required")]
    IntegrationNameRequired,
    #[error("integration name already exists")]
    DuplicateIntegrationName,
    #[error("database error: {0}")]
    Database(#[from] anyhow::Error),
    #[error("{0}")]
    InvalidInput(String),
}
impl StoreError {
    /// Whether this error means "no such row" (rendered as a themed 404 or a
    /// 404 JSON response rather than an error body).
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::BillNotFound
                | Self::ExpenseNotFound
                | Self::IntegrationNotFound
                | Self::ReceiptNotFound
        )
    }
}
impl From<sqlx::Error> for StoreError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.into())
    }
}

/// HTTP status of this error's JSON response.
#[must_use]
pub fn store_error_status(err: &StoreError) -> StatusCode {
    match err {
        StoreError::BillNotFound
        | StoreError::IntegrationNotFound
        | StoreError::ExpenseNotFound
        | StoreError::ReceiptNotFound => StatusCode::NOT_FOUND,
        StoreError::VendorRequired
        | StoreError::BillNeedsLineItems
        | StoreError::ScanUriRequired
        | StoreError::InvalidQuantity
        | StoreError::ExpenseDescriptionRequired
        | StoreError::InvalidAmount
        | StoreError::LinkedBillNotFound
        | StoreError::ReceiptChargeRequired
        | StoreError::ReceiptUserRequired
        | StoreError::IntegrationNameRequired
        | StoreError::OrderNotFound
        | StoreError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        StoreError::DuplicateIntegrationName => StatusCode::CONFLICT,
        StoreError::Orders(_) | StoreError::Payments(_) => StatusCode::BAD_GATEWAY,
        StoreError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_mapping_preserves_the_api_contract() {
        assert_eq!(
            store_error_status(&StoreError::BillNotFound),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            store_error_status(&StoreError::VendorRequired),
            StatusCode::BAD_REQUEST
        );
        // A duplicate integration name is a conflict, not a bad request.
        assert_eq!(
            store_error_status(&StoreError::DuplicateIntegrationName),
            StatusCode::CONFLICT
        );
        // An unreachable orders service is an upstream failure, not ours.
        assert_eq!(
            store_error_status(&StoreError::Orders("connection refused".to_string())),
            StatusCode::BAD_GATEWAY
        );
        assert_eq!(
            store_error_status(&StoreError::Database(anyhow::anyhow!("boom"))),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn only_missing_row_errors_are_not_found() {
        assert!(StoreError::BillNotFound.is_not_found());
        assert!(StoreError::ExpenseNotFound.is_not_found());
        assert!(StoreError::IntegrationNotFound.is_not_found());
        assert!(!StoreError::LinkedBillNotFound.is_not_found());
        assert!(!StoreError::OrderNotFound.is_not_found());
    }
}
