//! [`Integration`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sigma_pg::form::empty_to_none;

use super::{CreateIntegration, IntegrationProvider, UpdateIntegration};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Integration {
    pub id: String,
    pub name: String,
    pub provider: IntegrationProvider,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub updated_at: DateTime<Utc>,
}
impl Integration {
    /// New Integration from a create request.
    pub fn new(input: CreateIntegration) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: input.name.trim().to_string(),
            provider: input.provider,
            enabled: input.enabled.unwrap_or(true),
            external_account_id: input.external_account_id.and_then(empty_to_none),
            webhook_url: input.webhook_url.and_then(empty_to_none),
            notes: input.notes.and_then(empty_to_none),
            updated_at: Utc::now(),
        }
    }

    /// Apply a partial update in place.
    pub fn apply_update(&mut self, input: UpdateIntegration) {
        self.name = input.name.trim().to_string();
        self.provider = input.provider;
        self.enabled = input.enabled;
        self.external_account_id = input.external_account_id.and_then(empty_to_none);
        self.webhook_url = input.webhook_url.and_then(empty_to_none);
        self.notes = input.notes.and_then(empty_to_none);
        self.updated_at = Utc::now();
    }
}
