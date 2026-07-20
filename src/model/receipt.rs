//! [`Receipt`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sigma_pg::form::empty_to_none;

use super::{CreateReceipt, ReceiptKind, default_currency, normalize_currency};

/// Money received from a customer — one row per successful payments charge.
/// `charge_id` and `order_id` are cross-service references stored as opaque
/// ids, never database foreign keys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub charge_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    pub user_id: String,
    pub kind: ReceiptKind,
    pub amount_cents: i64,
    #[serde(default = "default_currency")]
    pub currency: String,
    pub occurred_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub updated_at: DateTime<Utc>,
}
impl Receipt {
    /// New Receipt from a create request.
    pub fn new(input: CreateReceipt) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            charge_id: input.charge_id.trim().to_string(),
            order_id: input.order_id.and_then(empty_to_none),
            user_id: input.user_id.trim().to_string(),
            kind: input.kind,
            amount_cents: input.amount_cents,
            currency: input
                .currency
                .map(normalize_currency)
                .unwrap_or_else(default_currency),
            occurred_at: input.occurred_at.unwrap_or(now),
            notes: input.notes.and_then(empty_to_none),
            updated_at: now,
        }
    }
}
