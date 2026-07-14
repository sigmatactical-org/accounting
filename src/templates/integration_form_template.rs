//! [`IntegrationFormTemplate`].

#[allow(unused_imports)]
use super::*;
use crate::model::Integration;
use askama::Template;
use sigma_theme::nav::SiteHeader;

#[derive(Template)]
#[template(path = "integration_form.html")]
pub(crate) struct IntegrationFormTemplate {
    pub(crate) integration: Option<Integration>,
    pub(crate) name: String,
    pub(crate) provider_quickbooks: bool,
    pub(crate) provider_xero: bool,
    pub(crate) provider_custom: bool,
    pub(crate) enabled: bool,
    pub(crate) external_account_id: String,
    pub(crate) webhook_url: String,
    pub(crate) notes: String,
    pub(crate) error: Option<String>,
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) copyright_years: String,
}
