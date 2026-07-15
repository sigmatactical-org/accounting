//! [`ExpenseFormValues`].

#[allow(unused_imports)]
use super::*;

/// Prefilled field values for the edit/create form.
pub struct ExpenseFormValues {
    pub expense_date: String,
    pub category: String,
    pub description: String,
    pub vendor: String,
    pub amount_cents: String,
    pub currency: String,
    pub receipt_uri: String,
    pub bill_id: String,
    pub order_id: String,
    pub notes: String,
}
