//! [`ExpenseForm`].

use chrono::NaiveDate;
use serde::Deserialize;
use sigma_pg::form::empty_to_none;

use super::{CreateExpense, ExpenseCategory, UpdateExpense, normalize_currency, parse_date};

/// The fallible parts of the form, parsed before any field is moved out so a
/// rejected submission can hand the raw form back for re-display.
pub type ParsedExpense = (ExpenseCategory, NaiveDate, i64);

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
    /// Parse the fallible fields, borrowing the form so a rejected submission
    /// can still be handed back for re-display.
    pub fn validate(&self) -> Result<ParsedExpense, String> {
        Ok((
            self.category.parse()?,
            parse_date(&self.expense_date, "expense date")?,
            parse_amount_cents(&self.amount_cents)?,
        ))
    }

    /// Build a create request from the form and its [`validate`] output.
    ///
    /// [`validate`]: Self::validate
    #[must_use]
    pub fn into_create(self, parsed: ParsedExpense) -> CreateExpense {
        let (category, expense_date, amount_cents) = parsed;
        CreateExpense {
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
        }
    }

    /// Build an update request from the form and its [`validate`] output.
    ///
    /// [`validate`]: Self::validate
    #[must_use]
    pub fn into_update(self, parsed: ParsedExpense) -> UpdateExpense {
        let (category, expense_date, amount_cents) = parsed;
        UpdateExpense {
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
        }
    }
}

fn parse_amount_cents(value: &str) -> Result<i64, String> {
    value
        .trim()
        .parse()
        .map_err(|_| format!("invalid amount (cents): {value}"))
}
