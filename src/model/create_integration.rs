//! [`CreateIntegration`].

use serde::Deserialize;

use super::IntegrationProvider;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateIntegration {
    pub name: String,
    pub provider: IntegrationProvider,
    #[serde(default)]
    pub enabled: Option<bool>,
    pub external_account_id: Option<String>,
    pub webhook_url: Option<String>,
    pub notes: Option<String>,
}
