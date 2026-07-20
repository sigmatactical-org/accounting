//! [`ExpenseCategory`].

use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// What an expense was spent on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExpenseCategory {
    Materials,
    Shipping,
    Tooling,
    Software,
    Travel,
    Fees,
    Other,
}
impl ExpenseCategory {
    /// Every variant, in display order.
    pub const ALL: [Self; 7] = [
        Self::Materials,
        Self::Shipping,
        Self::Tooling,
        Self::Software,
        Self::Travel,
        Self::Fees,
        Self::Other,
    ];

    /// Stored/serialized wire value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Materials => "materials",
            Self::Shipping => "shipping",
            Self::Tooling => "tooling",
            Self::Software => "software",
            Self::Travel => "travel",
            Self::Fees => "fees",
            Self::Other => "other",
        }
    }

    /// Human-readable label for the UI.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Materials => "Materials",
            Self::Shipping => "Shipping",
            Self::Tooling => "Tooling",
            Self::Software => "Software",
            Self::Travel => "Travel",
            Self::Fees => "Fees",
            Self::Other => "Other",
        }
    }
}
impl FromStr for ExpenseCategory {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim().to_lowercase();
        Self::ALL
            .into_iter()
            .find(|category| category.as_str() == value)
            .ok_or_else(|| format!("invalid expense category: {value}"))
    }
}
