//! [`ReceiptKind`].

use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// What a received payment was for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReceiptKind {
    Deposit,
    Balance,
    Refund,
}
impl ReceiptKind {
    /// Every variant, in display order.
    pub const ALL: [Self; 3] = [Self::Deposit, Self::Balance, Self::Refund];

    /// Stored/serialized wire value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deposit => "deposit",
            Self::Balance => "balance",
            Self::Refund => "refund",
        }
    }

    /// Human-readable label for the UI.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Deposit => "Deposit",
            Self::Balance => "Balance",
            Self::Refund => "Refund",
        }
    }

    /// Sign this kind contributes to a money-in total: refunds pay money back
    /// out, deposits and balance payments bring it in.
    #[must_use]
    pub fn sign(self) -> i64 {
        match self {
            Self::Deposit | Self::Balance => 1,
            Self::Refund => -1,
        }
    }
}
impl FromStr for ReceiptKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim().to_lowercase();
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == value)
            .ok_or_else(|| format!("invalid receipt kind: {value}"))
    }
}
