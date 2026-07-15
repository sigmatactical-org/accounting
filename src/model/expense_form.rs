//! [`ExpenseForm`].

#[allow(unused_imports)]
use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ExpenseForm {
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
impl ExpenseForm {
    /// Validate the form into a create request.
    pub fn into_create(self) -> Result<CreateExpense, String> {
        Ok(CreateExpense {
            expense_date: self.expense_date,
            category: parse_expense_category(&self.category)?,
            description: self.description,
            vendor: empty_to_none(self.vendor),
            amount_cents: parse_amount_cents(&self.amount_cents)?,
            currency: empty_to_none(self.currency),
            receipt_uri: empty_to_none(self.receipt_uri),
            bill_id: empty_to_none(self.bill_id),
            order_id: empty_to_none(self.order_id),
            notes: empty_to_none(self.notes),
        })
    }

    /// Validate the form into an update request.
    pub fn into_update(self) -> Result<UpdateExpense, String> {
        Ok(UpdateExpense {
            expense_date: self.expense_date,
            category: parse_expense_category(&self.category)?,
            description: self.description,
            vendor: empty_to_none(self.vendor),
            amount_cents: parse_amount_cents(&self.amount_cents)?,
            currency: normalize_currency(self.currency),
            receipt_uri: empty_to_none(self.receipt_uri),
            bill_id: empty_to_none(self.bill_id),
            order_id: empty_to_none(self.order_id),
            notes: empty_to_none(self.notes),
        })
    }
}

fn parse_amount_cents(value: &str) -> Result<i64, String> {
    value
        .trim()
        .parse()
        .map_err(|_| format!("invalid amount (cents): {value}"))
}
