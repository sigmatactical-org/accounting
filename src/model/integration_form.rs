//! [`IntegrationForm`].

use serde::Deserialize;
use sigma_pg::form::empty_to_none;

use super::{CreateIntegration, IntegrationProvider, UpdateIntegration};

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
    fn parse(&self) -> Result<IntegrationProvider, String> {
        self.provider.parse()
    }

    /// Validate the form into a create request, returning the rejection
    /// message and the untouched form when validation fails.
    pub fn into_create(self) -> Result<CreateIntegration, (String, Self)> {
        let provider = match self.parse() {
            Ok(provider) => provider,
            Err(message) => return Err((message, self)),
        };
        Ok(CreateIntegration {
            name: self.name,
            provider,
            enabled: Some(self.enabled.is_some()),
            external_account_id: empty_to_none(self.external_account_id),
            webhook_url: empty_to_none(self.webhook_url),
            notes: empty_to_none(self.notes),
        })
    }

    /// Validate the form into an update request, returning the rejection
    /// message and the untouched form when validation fails.
    pub fn into_update(self) -> Result<UpdateIntegration, (String, Self)> {
        let provider = match self.parse() {
            Ok(provider) => provider,
            Err(message) => return Err((message, self)),
        };
        Ok(UpdateIntegration {
            name: self.name,
            provider,
            enabled: self.enabled.is_some(),
            external_account_id: empty_to_none(self.external_account_id),
            webhook_url: empty_to_none(self.webhook_url),
            notes: empty_to_none(self.notes),
        })
    }
}
