//! [`CreateExpense`].

use chrono::NaiveDate;
use serde::Deserialize;

use super::ExpenseCategory;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateExpense {
    pub expense_date: NaiveDate,
    pub category: ExpenseCategory,
    pub description: String,
    #[serde(default)]
    pub vendor: Option<String>,
    pub amount_cents: i64,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub receipt_uri: Option<String>,
    #[serde(default)]
    pub bill_id: Option<String>,
    #[serde(default)]
    pub order_id: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}
