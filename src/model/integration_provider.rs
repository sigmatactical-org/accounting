//! [`IntegrationProvider`].

use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Accounting system an integration talks to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IntegrationProvider {
    QuickBooks,
    Xero,
    Custom,
}
impl IntegrationProvider {
    /// Every variant, in display order.
    pub const ALL: [Self; 3] = [Self::QuickBooks, Self::Xero, Self::Custom];

    /// Stored/serialized wire value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::QuickBooks => "quickbooks",
            Self::Xero => "xero",
            Self::Custom => "custom",
        }
    }

    /// Human-readable label for the UI.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::QuickBooks => "QuickBooks",
            Self::Xero => "Xero",
            Self::Custom => "Custom",
        }
    }
}
impl FromStr for IntegrationProvider {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim().to_lowercase();
        Self::ALL
            .into_iter()
            .find(|provider| provider.as_str() == value)
            .ok_or_else(|| format!("invalid integration provider: {value}"))
    }
}
