//! [`Expense`].

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sigma_pg::form::empty_to_none;

use super::{CreateExpense, ExpenseCategory, UpdateExpense, default_currency, normalize_currency};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expense {
    pub id: String,
    pub expense_date: NaiveDate,
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
    pub updated_at: DateTime<Utc>,
}
impl Expense {
    /// New Expense from a create request.
    pub fn new(input: CreateExpense) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            expense_date: input.expense_date,
            category: input.category,
            description: input.description.trim().to_string(),
            vendor: input.vendor.and_then(empty_to_none),
            amount_cents: input.amount_cents,
            currency: input
                .currency
                .map(normalize_currency)
                .unwrap_or_else(default_currency),
            receipt_uri: input.receipt_uri.and_then(empty_to_none),
            bill_id: input.bill_id.and_then(empty_to_none),
            order_id: input.order_id.and_then(empty_to_none),
            notes: input.notes.and_then(empty_to_none),
            updated_at: Utc::now(),
        }
    }

    /// Apply a partial update in place.
    pub fn apply_update(&mut self, input: UpdateExpense) {
        self.expense_date = input.expense_date;
        self.category = input.category;
        self.description = input.description.trim().to_string();
        self.vendor = input.vendor.and_then(empty_to_none);
        self.amount_cents = input.amount_cents;
        self.currency = normalize_currency(input.currency);
        self.receipt_uri = input.receipt_uri.and_then(empty_to_none);
        self.bill_id = input.bill_id.and_then(empty_to_none);
        self.order_id = input.order_id.and_then(empty_to_none);
        self.notes = input.notes.and_then(empty_to_none);
        self.updated_at = Utc::now();
    }
}
