//! [`BillForm`].

use chrono::NaiveDate;
use serde::Deserialize;
use sigma_pg::form::empty_to_none;

use super::{
    BillKind, BillLineItem, BillStatus, CreateBill, UpdateBill, normalize_currency, parse_date,
    parse_line_items_text, parse_optional_date,
};

/// The fallible parts of the form, parsed before any field is moved out so a
/// rejected submission can hand the raw form back for re-display.
type ParsedBill = (
    BillKind,
    BillStatus,
    NaiveDate,
    Option<NaiveDate>,
    Vec<BillLineItem>,
);

#[derive(Debug, Clone, Deserialize)]
pub struct BillForm {
    pub kind: String,
    pub status: String,
    pub vendor: String,
    pub invoice_number: String,
    #[serde(default)]
    pub order_id: String,
    pub bill_date: String,
    pub due_date: String,
    pub currency: String,
    pub line_items: String,
    pub scan_uri: String,
    pub notes: String,
}
impl BillForm {
    fn parse(&self) -> Result<ParsedBill, String> {
        Ok((
            self.kind.parse()?,
            self.status.parse()?,
            parse_date(&self.bill_date, "bill date")?,
            parse_optional_date(&self.due_date, "due date")?,
            parse_line_items_text(&self.line_items)?,
        ))
    }

    /// Validate the form into a create request, returning the rejection
    /// message and the untouched form when validation fails.
    pub fn into_create(self) -> Result<CreateBill, Box<(String, Self)>> {
        let (kind, status, bill_date, due_date, line_items) = match self.parse() {
            Ok(parsed) => parsed,
            Err(message) => return Err(Box::new((message, self))),
        };
        Ok(CreateBill {
            kind,
            status: Some(status),
            vendor: self.vendor,
            invoice_number: empty_to_none(self.invoice_number),
            order_id: empty_to_none(self.order_id),
            bill_date,
            due_date,
            currency: empty_to_none(self.currency),
            line_items,
            scan_uri: empty_to_none(self.scan_uri),
            notes: empty_to_none(self.notes),
        })
    }

    /// Validate the form into an update request, returning the rejection
    /// message and the untouched form when validation fails.
    pub fn into_update(self) -> Result<UpdateBill, Box<(String, Self)>> {
        let (kind, status, bill_date, due_date, line_items) = match self.parse() {
            Ok(parsed) => parsed,
            Err(message) => return Err(Box::new((message, self))),
        };
        Ok(UpdateBill {
            kind,
            status,
            vendor: self.vendor,
            invoice_number: empty_to_none(self.invoice_number),
            order_id: empty_to_none(self.order_id),
            bill_date,
            due_date,
            currency: normalize_currency(self.currency),
            line_items,
            scan_uri: empty_to_none(self.scan_uri),
            notes: empty_to_none(self.notes),
        })
    }
}
