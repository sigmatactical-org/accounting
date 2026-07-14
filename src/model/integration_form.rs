//! [`IntegrationForm`].

#[allow(unused_imports)]
use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationForm {
    pub name: String,
    pub provider: String,
    pub enabled: Option<String>,
    pub external_account_id: String,
    pub webhook_url: String,
    pub notes: String,
}
impl IntegrationForm {
    /// Validate the form into a create request.
    pub fn into_create(self) -> Result<CreateIntegration, String> {
        Ok(CreateIntegration {
            name: self.name,
            provider: parse_integration_provider(&self.provider)?,
            enabled: Some(self.enabled.is_some()),
            external_account_id: empty_to_none(self.external_account_id),
            webhook_url: empty_to_none(self.webhook_url),
            notes: empty_to_none(self.notes),
        })
    }

    /// Validate the form into an update request.
    pub fn into_update(self) -> Result<UpdateIntegration, String> {
        Ok(UpdateIntegration {
            name: self.name,
            provider: parse_integration_provider(&self.provider)?,
            enabled: self.enabled.is_some(),
            external_account_id: empty_to_none(self.external_account_id),
            webhook_url: empty_to_none(self.webhook_url),
            notes: empty_to_none(self.notes),
        })
    }
}
