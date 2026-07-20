//! [`IntegrationFormValues`].

use crate::model::IntegrationForm;

/// Prefilled field values for the edit/create form.
pub struct IntegrationFormValues {
    pub name: String,
    pub provider: String,
    pub enabled: bool,
    pub external_account_id: String,
    pub webhook_url: String,
    pub notes: String,
}
impl From<IntegrationForm> for IntegrationFormValues {
    /// Re-display exactly what the user submitted (rejected-input path).
    fn from(form: IntegrationForm) -> Self {
        Self {
            name: form.name,
            provider: form.provider,
            enabled: form.enabled.is_some(),
            external_account_id: form.external_account_id,
            webhook_url: form.webhook_url,
            notes: form.notes,
        }
    }
}
