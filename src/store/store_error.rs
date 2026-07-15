//! [`StoreError`].

#[allow(unused_imports)]
use super::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("bill not found")]
    BillNotFound,
    #[error("integration not found")]
    IntegrationNotFound,
    #[error("vendor is required")]
    VendorRequired,
    #[error("bill date is required")]
    BillDateRequired,
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
    #[error("expense date is required")]
    ExpenseDateRequired,
    #[error("expense amount must be at least 1 cent")]
    InvalidAmount,
    #[error("linked bill not found")]
    LinkedBillNotFound,
    #[error("linked order not found")]
    OrderNotFound,
    #[error("orders service error: {0}")]
    Orders(String),
    #[error("integration name is required")]
    IntegrationNameRequired,
    #[error("integration name already exists")]
    DuplicateIntegrationName,
    #[error("database error: {0}")]
    Database(#[from] anyhow::Error),
    #[error("{0}")]
    InvalidInput(String),
}
impl From<sqlx::Error> for StoreError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.into())
    }
}
