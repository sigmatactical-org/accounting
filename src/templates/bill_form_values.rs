//! [`BillFormValues`].

#[allow(unused_imports)]
use super::*;

/// Prefilled field values for the edit/create form.
pub struct BillFormValues {
    pub kind: String,
    pub status: String,
    pub vendor: String,
    pub invoice_number: String,
    pub order_id: String,
    pub bill_date: String,
    pub due_date: String,
    pub currency: String,
    pub line_items: String,
    pub scan_uri: String,
    pub notes: String,
}
