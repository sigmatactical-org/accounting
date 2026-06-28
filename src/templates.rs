use askama::Template;

use crate::catalog::CatalogSku;
use crate::model::{
    Bill, BillKind, BillStatus, Integration, IntegrationProvider, format_line_items_text,
};
use sigma_theme::copyright_years;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    bills: Vec<BillRow>,
    integrations: Vec<IntegrationRow>,
    catalog_skus: Vec<CatalogSkuRef>,
    catalog_notice: Option<String>,
    catalog_configured: bool,
    message: Option<String>,
    copyright_years: String,
}

#[derive(Template)]
#[template(path = "bill_form.html")]
struct BillFormTemplate {
    bill: Option<Bill>,
    kind_scanned: bool,
    kind_digital: bool,
    status_draft: bool,
    status_approved: bool,
    status_paid: bool,
    status_void: bool,
    vendor: String,
    invoice_number: String,
    bill_date: String,
    due_date: String,
    currency: String,
    line_items: String,
    scan_uri: String,
    notes: String,
    catalog_skus: Vec<CatalogSkuRef>,
    error: Option<String>,
    copyright_years: String,
}

#[derive(Template)]
#[template(path = "integration_form.html")]
struct IntegrationFormTemplate {
    integration: Option<Integration>,
    name: String,
    provider_quickbooks: bool,
    provider_xero: bool,
    provider_custom: bool,
    enabled: bool,
    external_account_id: String,
    webhook_url: String,
    notes: String,
    error: Option<String>,
    copyright_years: String,
}

pub struct BillRow {
    pub bill: Bill,
    pub kind_label: String,
    pub status_label: String,
    pub total_display: String,
}

pub struct IntegrationRow {
    pub integration: Integration,
    pub provider_label: String,
}

pub struct CatalogSkuRef {
    pub id: String,
    pub sku_code: String,
    pub name: String,
}

pub struct BillFormValues {
    pub kind: String,
    pub status: String,
    pub vendor: String,
    pub invoice_number: String,
    pub bill_date: String,
    pub due_date: String,
    pub currency: String,
    pub line_items: String,
    pub scan_uri: String,
    pub notes: String,
}

pub struct IntegrationFormValues {
    pub name: String,
    pub provider: String,
    pub enabled: bool,
    pub external_account_id: String,
    pub webhook_url: String,
    pub notes: String,
}

fn bill_rows(bills: Vec<Bill>) -> Vec<BillRow> {
    bills
        .into_iter()
        .map(|bill| {
            let kind_label = match bill.kind {
                BillKind::Scanned => "Scanned".to_string(),
                BillKind::Digital => "Digital".to_string(),
            };
            let status_label = match bill.status {
                BillStatus::Draft => "Draft".to_string(),
                BillStatus::Approved => "Approved".to_string(),
                BillStatus::Paid => "Paid".to_string(),
                BillStatus::Void => "Void".to_string(),
            };
            let total_display = format_amount(bill.total_cents, &bill.currency);
            BillRow {
                bill,
                kind_label,
                status_label,
                total_display,
            }
        })
        .collect()
}

fn integration_rows(integrations: Vec<Integration>) -> Vec<IntegrationRow> {
    integrations
        .into_iter()
        .map(|integration| {
            let provider_label = match integration.provider {
                IntegrationProvider::QuickBooks => "QuickBooks".to_string(),
                IntegrationProvider::Xero => "Xero".to_string(),
                IntegrationProvider::Custom => "Custom".to_string(),
            };
            IntegrationRow {
                integration,
                provider_label,
            }
        })
        .collect()
}

fn catalog_sku_refs(skus: Vec<CatalogSku>) -> Vec<CatalogSkuRef> {
    skus.into_iter()
        .map(|sku| CatalogSkuRef {
            id: sku.id,
            sku_code: sku.sku_code,
            name: sku.name,
        })
        .collect()
}

fn format_amount(cents: i64, currency: &str) -> String {
    let dollars = cents as f64 / 100.0;
    format!("{currency} {dollars:.2}")
}

fn values_from_bill(bill: &Bill) -> BillFormValues {
    BillFormValues {
        kind: match bill.kind {
            BillKind::Scanned => "scanned".to_string(),
            BillKind::Digital => "digital".to_string(),
        },
        status: match bill.status {
            BillStatus::Draft => "draft".to_string(),
            BillStatus::Approved => "approved".to_string(),
            BillStatus::Paid => "paid".to_string(),
            BillStatus::Void => "void".to_string(),
        },
        vendor: bill.vendor.clone(),
        invoice_number: bill.invoice_number.clone().unwrap_or_default(),
        bill_date: bill.bill_date.clone(),
        due_date: bill.due_date.clone().unwrap_or_default(),
        currency: bill.currency.clone(),
        line_items: format_line_items_text(&bill.line_items),
        scan_uri: bill.scan_uri.clone().unwrap_or_default(),
        notes: bill.notes.clone().unwrap_or_default(),
    }
}

fn default_bill_form_values() -> BillFormValues {
    BillFormValues {
        kind: "digital".to_string(),
        status: "draft".to_string(),
        vendor: String::new(),
        invoice_number: String::new(),
        bill_date: String::new(),
        due_date: String::new(),
        currency: "USD".to_string(),
        line_items: String::new(),
        scan_uri: String::new(),
        notes: String::new(),
    }
}

fn values_from_integration(integration: &Integration) -> IntegrationFormValues {
    IntegrationFormValues {
        name: integration.name.clone(),
        provider: match integration.provider {
            IntegrationProvider::QuickBooks => "quickbooks".to_string(),
            IntegrationProvider::Xero => "xero".to_string(),
            IntegrationProvider::Custom => "custom".to_string(),
        },
        enabled: integration.enabled,
        external_account_id: integration.external_account_id.clone().unwrap_or_default(),
        webhook_url: integration.webhook_url.clone().unwrap_or_default(),
        notes: integration.notes.clone().unwrap_or_default(),
    }
}

fn default_integration_form_values() -> IntegrationFormValues {
    IntegrationFormValues {
        name: String::new(),
        provider: "quickbooks".to_string(),
        enabled: true,
        external_account_id: String::new(),
        webhook_url: String::new(),
        notes: String::new(),
    }
}

fn render_bill_form(
    catalog_skus: Vec<CatalogSku>,
    bill: Option<Bill>,
    error: Option<String>,
    values: BillFormValues,
) -> Result<String, askama::Error> {
    let kind = values.kind.to_lowercase();
    let status = values.status.to_lowercase();
    BillFormTemplate {
        bill,
        kind_scanned: kind == "scanned",
        kind_digital: kind == "digital",
        status_draft: status == "draft",
        status_approved: status == "approved",
        status_paid: status == "paid",
        status_void: status == "void",
        vendor: values.vendor,
        invoice_number: values.invoice_number,
        bill_date: values.bill_date,
        due_date: values.due_date,
        currency: values.currency,
        line_items: values.line_items,
        scan_uri: values.scan_uri,
        notes: values.notes,
        catalog_skus: catalog_sku_refs(catalog_skus),
        error,
        copyright_years: copyright_years(),
    }
    .render()
}

fn render_integration_form(
    integration: Option<Integration>,
    error: Option<String>,
    values: IntegrationFormValues,
) -> Result<String, askama::Error> {
    let provider = values.provider.to_lowercase();
    IntegrationFormTemplate {
        integration,
        name: values.name,
        provider_quickbooks: provider == "quickbooks",
        provider_xero: provider == "xero",
        provider_custom: provider == "custom",
        enabled: values.enabled,
        external_account_id: values.external_account_id,
        webhook_url: values.webhook_url,
        notes: values.notes,
        error,
        copyright_years: copyright_years(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_index_html(
    bills: Vec<Bill>,
    integrations: Vec<Integration>,
    catalog_skus: Vec<CatalogSku>,
    catalog_notice: Option<String>,
    message: Option<String>,
) -> Result<String, askama::Error> {
    IndexTemplate {
        bills: bill_rows(bills),
        integrations: integration_rows(integrations),
        catalog_skus: catalog_sku_refs(catalog_skus),
        catalog_notice,
        catalog_configured: crate::config::catalog_configured(),
        message,
        copyright_years: copyright_years(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_bill_form_html(
    catalog_skus: Vec<CatalogSku>,
    bill: Option<Bill>,
    error: Option<String>,
) -> Result<String, askama::Error> {
    let values = bill
        .as_ref()
        .map(values_from_bill)
        .unwrap_or_else(default_bill_form_values);
    render_bill_form(catalog_skus, bill, error, values)
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_bill_form_html_with_values(
    catalog_skus: Vec<CatalogSku>,
    bill: Option<Bill>,
    error: Option<String>,
    values: BillFormValues,
) -> Result<String, askama::Error> {
    render_bill_form(catalog_skus, bill, error, values)
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_integration_form_html(
    integration: Option<Integration>,
    error: Option<String>,
) -> Result<String, askama::Error> {
    let values = integration
        .as_ref()
        .map(values_from_integration)
        .unwrap_or_else(default_integration_form_values);
    render_integration_form(integration, error, values)
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_integration_form_html_with_values(
    integration: Option<Integration>,
    error: Option<String>,
    values: IntegrationFormValues,
) -> Result<String, askama::Error> {
    render_integration_form(integration, error, values)
}
