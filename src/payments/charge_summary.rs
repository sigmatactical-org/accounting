//! [`ChargeSummary`].

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// One charge as listed by the payments service (`GET /api/charges`).
/// Deserialized loosely: accounting only needs the fields it turns into a
/// receipt, and ignores the rest of the payments payload.
#[derive(Debug, Clone, Deserialize)]
pub struct ChargeSummary {
    pub id: String,
    pub user_id: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
impl ChargeSummary {
    /// Whether this charge actually took money (only those become receipts).
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.status == "succeeded"
    }
}
