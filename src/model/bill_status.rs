//! [`BillStatus`].

use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Where a bill sits in the approve/pay workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BillStatus {
    Draft,
    Approved,
    Paid,
    Void,
}
impl BillStatus {
    /// Every variant, in display order.
    pub const ALL: [Self; 4] = [Self::Draft, Self::Approved, Self::Paid, Self::Void];

    /// Stored/serialized wire value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Approved => "approved",
            Self::Paid => "paid",
            Self::Void => "void",
        }
    }

    /// Human-readable label for the UI.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Approved => "Approved",
            Self::Paid => "Paid",
            Self::Void => "Void",
        }
    }
}
impl FromStr for BillStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim().to_lowercase();
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == value)
            .ok_or_else(|| format!("invalid bill status: {value}"))
    }
}
