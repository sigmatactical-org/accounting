//! [`Integration`].

#[allow(unused_imports)]
use super::*;
use serde::{Deserialize, Serialize};

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
    pub updated_at: String,
}
impl Integration {
    /// New Integration from a create request.
    pub fn new(input: CreateIntegration) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: input.name.trim().to_string(),
            provider: input.provider,
            enabled: input.enabled.unwrap_or(true),
            external_account_id: input.external_account_id.map(|s| s.trim().to_string()),
            webhook_url: input.webhook_url.map(|s| s.trim().to_string()),
            notes: input.notes.map(|s| s.trim().to_string()),
            updated_at: now,
        }
    }

    /// Apply a partial update in place.
    pub fn apply_update(&mut self, input: UpdateIntegration) {
        self.name = input.name.trim().to_string();
        self.provider = input.provider;
        self.enabled = input.enabled;
        self.external_account_id = input.external_account_id.map(|s| s.trim().to_string());
        self.webhook_url = input.webhook_url.map(|s| s.trim().to_string());
        self.notes = input.notes.map(|s| s.trim().to_string());
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}
