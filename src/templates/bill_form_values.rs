//! [`BillFormValues`].

use crate::model::BillForm;

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
impl From<BillForm> for BillFormValues {
    /// Re-display exactly what the user submitted (rejected-input path).
    fn from(form: BillForm) -> Self {
        Self {
            kind: form.kind,
            status: form.status,
            vendor: form.vendor,
            invoice_number: form.invoice_number,
            order_id: form.order_id,
            bill_date: form.bill_date,
            due_date: form.due_date,
            currency: form.currency,
            line_items: form.line_items,
            scan_uri: form.scan_uri,
            notes: form.notes,
        }
    }
}
