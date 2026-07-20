//! [`BillKind`].

use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// How a bill entered the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BillKind {
    Scanned,
    Digital,
}
impl BillKind {
    /// Every variant, in display order.
    pub const ALL: [Self; 2] = [Self::Scanned, Self::Digital];

    /// Stored/serialized wire value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scanned => "scanned",
            Self::Digital => "digital",
        }
    }

    /// Human-readable label for the UI.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Scanned => "Scanned",
            Self::Digital => "Digital",
        }
    }
}
impl FromStr for BillKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim().to_lowercase();
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == value)
            .ok_or_else(|| format!("invalid bill kind: {value}"))
    }
}
