//! [`UpdateIntegration`].

#[allow(unused_imports)]
use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateIntegration {
    pub name: String,
    pub provider: IntegrationProvider,
    pub enabled: bool,
    pub external_account_id: Option<String>,
    pub webhook_url: Option<String>,
    pub notes: Option<String>,
}
