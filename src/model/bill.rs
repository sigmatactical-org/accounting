//! [`Bill`].

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sigma_pg::form::empty_to_none;

use super::{
    BillKind, BillLineItem, BillStatus, CreateBill, UpdateBill, compute_total_cents,
    default_currency, normalize_currency,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bill {
    pub id: String,
    pub kind: BillKind,
    pub status: BillStatus,
    pub vendor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    pub bill_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<NaiveDate>,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub line_items: Vec<BillLineItem>,
    pub total_cents: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub updated_at: DateTime<Utc>,
}
impl Bill {
    /// New Bill from a create request.
    pub fn new(input: CreateBill) -> Self {
        let total_cents = compute_total_cents(&input.line_items);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            kind: input.kind,
            status: input.status.unwrap_or(BillStatus::Draft),
            vendor: input.vendor.trim().to_string(),
            invoice_number: input.invoice_number.and_then(empty_to_none),
            order_id: input.order_id.and_then(empty_to_none),
            bill_date: input.bill_date,
            due_date: input.due_date,
            currency: input
                .currency
                .map(normalize_currency)
                .unwrap_or_else(default_currency),
            line_items: input.line_items,
            total_cents,
            scan_uri: input.scan_uri.and_then(empty_to_none),
            notes: input.notes.and_then(empty_to_none),
            updated_at: Utc::now(),
        }
    }

    /// Apply a partial update in place.
    pub fn apply_update(&mut self, input: UpdateBill) {
        self.kind = input.kind;
        self.status = input.status;
        self.vendor = input.vendor.trim().to_string();
        self.invoice_number = input.invoice_number.and_then(empty_to_none);
        self.order_id = input.order_id.and_then(empty_to_none);
        self.bill_date = input.bill_date;
        self.due_date = input.due_date;
        self.currency = normalize_currency(input.currency);
        self.line_items = input.line_items;
        self.total_cents = compute_total_cents(&self.line_items);
        self.scan_uri = input.scan_uri.and_then(empty_to_none);
        self.notes = input.notes.and_then(empty_to_none);
        self.updated_at = Utc::now();
    }
}
