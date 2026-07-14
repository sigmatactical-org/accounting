//! [`BillLineItem`].

#[allow(unused_imports)]
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BillLineItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku_id: Option<String>,
    pub description: String,
    pub quantity: u32,
    pub unit_price_cents: i64,
}
