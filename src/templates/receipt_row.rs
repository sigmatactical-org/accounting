//! [`ReceiptRow`].

use crate::model::Receipt;

/// One rendered table row.
pub struct ReceiptRow {
    pub receipt: Receipt,
    pub kind_label: &'static str,
    pub amount_display: String,
    pub occurred_display: String,
    /// Link to the linked order's admin page, when `ACCOUNTING_ORDERS_PUBLIC_URL` is set.
    pub order_href: Option<String>,
}
