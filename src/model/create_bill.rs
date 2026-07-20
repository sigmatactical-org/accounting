//! [`CreateBill`].

use chrono::NaiveDate;
use serde::Deserialize;

use super::{BillKind, BillLineItem, BillStatus};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBill {
    pub kind: BillKind,
    #[serde(default)]
    pub status: Option<BillStatus>,
    pub vendor: String,
    pub invoice_number: Option<String>,
    #[serde(default)]
    pub order_id: Option<String>,
    pub bill_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub line_items: Vec<BillLineItem>,
    pub scan_uri: Option<String>,
    pub notes: Option<String>,
}
