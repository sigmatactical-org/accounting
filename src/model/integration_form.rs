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
    /// Parse the fallible fields, borrowing the form so a rejected submission
    /// can still be handed back for re-display.
    pub fn validate(&self) -> Result<IntegrationProvider, String> {
        self.provider.parse()
    }

    /// Build a create request from the form and its [`validate`] output.
    ///
    /// [`validate`]: Self::validate
    #[must_use]
    pub fn into_create(self, provider: IntegrationProvider) -> CreateIntegration {
        CreateIntegration {
            name: self.name,
            provider,
            enabled: Some(self.enabled.is_some()),
            external_account_id: empty_to_none(self.external_account_id),
            webhook_url: empty_to_none(self.webhook_url),
            notes: empty_to_none(self.notes),
        }
    }

    /// Build an update request from the form and its [`validate`] output.
    ///
    /// [`validate`]: Self::validate
    #[must_use]
    pub fn into_update(self, provider: IntegrationProvider) -> UpdateIntegration {
        UpdateIntegration {
            name: self.name,
            provider,
            enabled: self.enabled.is_some(),
            external_account_id: empty_to_none(self.external_account_id),
            webhook_url: empty_to_none(self.webhook_url),
            notes: empty_to_none(self.notes),
        }
    }
}
