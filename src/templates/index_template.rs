//! [`IndexTemplate`].

use askama::Template;
use sigma_theme::nav::SiteHeader;

use super::{BillRow, CatalogSkuRef, ExpenseRow, IntegrationRow, MoneySummary, ReceiptRow};

#[derive(Template)]
#[template(path = "index.html")]
pub(crate) struct IndexTemplate {
    pub(crate) bills: Vec<BillRow>,
    pub(crate) expenses: Vec<ExpenseRow>,
    pub(crate) expense_total: Option<String>,
    pub(crate) receipts: Vec<ReceiptRow>,
    pub(crate) money: MoneySummary,
    pub(crate) integrations: Vec<IntegrationRow>,
    pub(crate) catalog_skus: Vec<CatalogSkuRef>,
    pub(crate) catalog_notice: Option<String>,
    pub(crate) catalog_configured: bool,
    pub(crate) payments_configured: bool,
    pub(crate) message: Option<String>,
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) copyright_years: String,
}
