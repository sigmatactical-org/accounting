//! [`Bill`].

#[allow(unused_imports)]
use super::*;
use serde::{Deserialize, Serialize};

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
    pub bill_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub line_items: Vec<BillLineItem>,
    pub total_cents: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub updated_at: String,
}
impl Bill {
    /// New Bill from a create request.
    pub fn new(input: CreateBill) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let total_cents = compute_total_cents(&input.line_items);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            kind: input.kind,
            status: input.status.unwrap_or(BillStatus::Draft),
            vendor: input.vendor.trim().to_string(),
            invoice_number: input.invoice_number.map(|s| s.trim().to_string()),
            order_id: normalize_order_id(input.order_id),
            bill_date: input.bill_date.trim().to_string(),
            due_date: input.due_date.map(|s| s.trim().to_string()),
            currency: input
                .currency
                .map(|s| s.trim().to_uppercase())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(default_currency),
            line_items: input.line_items,
            total_cents,
            scan_uri: input.scan_uri.map(|s| s.trim().to_string()),
            notes: input.notes.map(|s| s.trim().to_string()),
            updated_at: now,
        }
    }

    /// Apply a partial update in place.
    pub fn apply_update(&mut self, input: UpdateBill) {
        self.kind = input.kind;
        self.status = input.status;
        self.vendor = input.vendor.trim().to_string();
        self.invoice_number = input.invoice_number.map(|s| s.trim().to_string());
        self.order_id = normalize_order_id(input.order_id);
        self.bill_date = input.bill_date.trim().to_string();
        self.due_date = input.due_date.map(|s| s.trim().to_string());
        self.currency = normalize_currency(input.currency);
        self.line_items = input.line_items;
        self.total_cents = compute_total_cents(&self.line_items);
        self.scan_uri = input.scan_uri.map(|s| s.trim().to_string());
        self.notes = input.notes.map(|s| s.trim().to_string());
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

fn normalize_order_id(order_id: Option<String>) -> Option<String> {
    order_id
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
