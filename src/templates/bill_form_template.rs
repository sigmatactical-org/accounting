//! [`BillFormTemplate`].

use crate::model::Bill;
use askama::Template;
use sigma_theme::nav::SiteHeader;

use super::CatalogSkuRef;

#[derive(Template)]
#[template(path = "bill_form.html")]
pub(crate) struct BillFormTemplate {
    pub(crate) bill: Option<Bill>,
    pub(crate) kind_scanned: bool,
    pub(crate) kind_digital: bool,
    pub(crate) status_draft: bool,
    pub(crate) status_approved: bool,
    pub(crate) status_paid: bool,
    pub(crate) status_void: bool,
    pub(crate) vendor: String,
    pub(crate) invoice_number: String,
    pub(crate) order_id: String,
    pub(crate) bill_date: String,
    pub(crate) due_date: String,
    pub(crate) currency: String,
    pub(crate) line_items: String,
    pub(crate) scan_uri: String,
    pub(crate) notes: String,
    pub(crate) catalog_skus: Vec<CatalogSkuRef>,
    pub(crate) error: Option<String>,
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) copyright_years: String,
}
