//! [`BillForm`].

#[allow(unused_imports)]
use super::*;
use serde::Deserialize;

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
    /// Validate the form into a create request.
    pub fn into_create(self) -> Result<CreateBill, String> {
        Ok(CreateBill {
            kind: parse_bill_kind(&self.kind)?,
            status: Some(parse_bill_status(&self.status)?),
            vendor: self.vendor,
            invoice_number: empty_to_none(self.invoice_number),
            order_id: empty_to_none(self.order_id),
            bill_date: self.bill_date,
            due_date: empty_to_none(self.due_date),
            currency: empty_to_none(self.currency),
            line_items: parse_line_items_text(&self.line_items)?,
            scan_uri: empty_to_none(self.scan_uri),
            notes: empty_to_none(self.notes),
        })
    }

    /// Validate the form into an update request.
    pub fn into_update(self) -> Result<UpdateBill, String> {
        Ok(UpdateBill {
            kind: parse_bill_kind(&self.kind)?,
            status: parse_bill_status(&self.status)?,
            vendor: self.vendor,
            invoice_number: empty_to_none(self.invoice_number),
            order_id: empty_to_none(self.order_id),
            bill_date: self.bill_date,
            due_date: empty_to_none(self.due_date),
            currency: normalize_currency(self.currency),
            line_items: parse_line_items_text(&self.line_items)?,
            scan_uri: empty_to_none(self.scan_uri),
            notes: empty_to_none(self.notes),
        })
    }
}
