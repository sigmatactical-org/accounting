//! [`ExpenseRow`].

use crate::model::Expense;

/// One rendered table row.
pub struct ExpenseRow {
    pub expense: Expense,
    pub category_label: &'static str,
    pub amount_display: String,
    /// Link to the linked order's admin page, when `ACCOUNTING_ORDERS_PUBLIC_URL` is set.
    pub order_href: Option<String>,
}
