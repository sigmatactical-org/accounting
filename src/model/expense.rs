//! [`Expense`].

#[allow(unused_imports)]
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expense {
    pub id: String,
    pub expense_date: String,
    pub category: ExpenseCategory,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    pub amount_cents: i64,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bill_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub updated_at: String,
}
impl Expense {
    /// New Expense from a create request.
    pub fn new(input: CreateExpense) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            expense_date: input.expense_date.trim().to_string(),
            category: input.category,
            description: input.description.trim().to_string(),
            vendor: trim_to_none(input.vendor),
            amount_cents: input.amount_cents,
            currency: input
                .currency
                .map(|s| s.trim().to_uppercase())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(default_currency),
            receipt_uri: trim_to_none(input.receipt_uri),
            bill_id: trim_to_none(input.bill_id),
            order_id: trim_to_none(input.order_id),
            notes: trim_to_none(input.notes),
            updated_at: now,
        }
    }

    /// Apply a partial update in place.
    pub fn apply_update(&mut self, input: UpdateExpense) {
        self.expense_date = input.expense_date.trim().to_string();
        self.category = input.category;
        self.description = input.description.trim().to_string();
        self.vendor = trim_to_none(input.vendor);
        self.amount_cents = input.amount_cents;
        self.currency = normalize_currency(input.currency);
        self.receipt_uri = trim_to_none(input.receipt_uri);
        self.bill_id = trim_to_none(input.bill_id);
        self.order_id = trim_to_none(input.order_id);
        self.notes = trim_to_none(input.notes);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

fn trim_to_none(value: Option<String>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
