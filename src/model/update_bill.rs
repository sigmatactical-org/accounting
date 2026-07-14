//! [`UpdateBill`].

#[allow(unused_imports)]
use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBill {
    pub kind: BillKind,
    pub status: BillStatus,
    pub vendor: String,
    pub invoice_number: Option<String>,
    pub bill_date: String,
    pub due_date: Option<String>,
    pub currency: String,
    #[serde(default)]
    pub line_items: Vec<BillLineItem>,
    pub scan_uri: Option<String>,
    pub notes: Option<String>,
}
