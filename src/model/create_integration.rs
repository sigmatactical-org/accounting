//! [`CreateIntegration`].

#[allow(unused_imports)]
use super::*;
use serde::Deserialize;

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
