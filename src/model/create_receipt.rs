//! [`CreateReceipt`].

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::ReceiptKind;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateReceipt {
    pub charge_id: String,
    #[serde(default)]
    pub order_id: Option<String>,
    pub user_id: String,
    pub kind: ReceiptKind,
    pub amount_cents: i64,
    #[serde(default)]
    pub currency: Option<String>,
    /// RFC 3339; defaults to now when omitted.
    #[serde(default)]
    pub occurred_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub notes: Option<String>,
}
