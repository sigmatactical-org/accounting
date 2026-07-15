//! [`ExpenseFormTemplate`].

#[allow(unused_imports)]
use super::*;
use crate::model::Expense;
use askama::Template;
use sigma_theme::nav::SiteHeader;

#[derive(Template)]
#[template(path = "expense_form.html")]
pub(crate) struct ExpenseFormTemplate {
    pub(crate) expense: Option<Expense>,
    pub(crate) expense_date: String,
    pub(crate) category_options: Vec<CategoryOption>,
    pub(crate) description: String,
    pub(crate) vendor: String,
    pub(crate) amount_cents: String,
    pub(crate) currency: String,
    pub(crate) receipt_uri: String,
    pub(crate) bill_id: String,
    pub(crate) order_id: String,
    pub(crate) notes: String,
    pub(crate) error: Option<String>,
    pub(crate) site_header: SiteHeader,
    pub(crate) site_nav: String,
    pub(crate) copyright_years: String,
}
