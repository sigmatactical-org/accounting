//! [`IntegrationRow`].

#[allow(unused_imports)]
use super::*;
use crate::model::Integration;

/// One rendered table row.
pub struct IntegrationRow {
    pub integration: Integration,
    pub provider_label: String,
}
