//! [`ExpenseForm`].

use chrono::NaiveDate;
use serde::Deserialize;
use sigma_pg::form::empty_to_none;

use super::{CreateExpense, ExpenseCategory, UpdateExpense, normalize_currency, parse_date};

/// The fallible parts of the form, parsed before any field is moved out so a
/// rejected submission can hand the raw form back for re-display.
type ParsedExpense = (ExpenseCategory, NaiveDate, i64);

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
    fn parse(&self) -> Result<ParsedExpense, String> {
        Ok((
            self.category.parse()?,
            parse_date(&self.expense_date, "expense date")?,
            parse_amount_cents(&self.amount_cents)?,
        ))
    }

    /// Validate the form into a create request, returning the rejection
    /// message and the untouched form when validation fails.
    pub fn into_create(self) -> Result<CreateExpense, (String, Self)> {
        let (category, expense_date, amount_cents) = match self.parse() {
            Ok(parsed) => parsed,
            Err(message) => return Err((message, self)),
        };
        Ok(CreateExpense {
            expense_date,
            category,
            description: self.description,
            vendor: empty_to_none(self.vendor),
            amount_cents,
            currency: empty_to_none(self.currency),
            receipt_uri: empty_to_none(self.receipt_uri),
            bill_id: empty_to_none(self.bill_id),
            order_id: empty_to_none(self.order_id),
            notes: empty_to_none(self.notes),
        })
    }

    /// Validate the form into an update request, returning the rejection
    /// message and the untouched form when validation fails.
    pub fn into_update(self) -> Result<UpdateExpense, (String, Self)> {
        let (category, expense_date, amount_cents) = match self.parse() {
            Ok(parsed) => parsed,
            Err(message) => return Err((message, self)),
        };
        Ok(UpdateExpense {
            expense_date,
            category,
            description: self.description,
            vendor: empty_to_none(self.vendor),
            amount_cents,
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
