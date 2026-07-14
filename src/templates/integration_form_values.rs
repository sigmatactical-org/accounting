//! [`IntegrationFormValues`].

#[allow(unused_imports)]
use super::*;

/// Prefilled field values for the edit/create form.
pub struct IntegrationFormValues {
    pub name: String,
    pub provider: String,
    pub enabled: bool,
    pub external_account_id: String,
    pub webhook_url: String,
    pub notes: String,
}
