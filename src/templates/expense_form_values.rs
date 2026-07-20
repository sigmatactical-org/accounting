//! [`ExpenseFormValues`].

use crate::model::ExpenseForm;

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
impl From<ExpenseForm> for ExpenseFormValues {
    /// Re-display exactly what the user submitted (rejected-input path).
    fn from(form: ExpenseForm) -> Self {
        Self {
            expense_date: form.expense_date,
            category: form.category,
            description: form.description,
            vendor: form.vendor,
            amount_cents: form.amount_cents,
            currency: form.currency,
            receipt_uri: form.receipt_uri,
            bill_id: form.bill_id,
            order_id: form.order_id,
            notes: form.notes,
        }
    }
}
