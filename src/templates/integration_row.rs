//! [`IntegrationRow`].

use crate::model::Integration;

/// One rendered table row.
pub struct IntegrationRow {
    pub integration: Integration,
    pub provider_label: &'static str,
    pub updated_display: String,
}
