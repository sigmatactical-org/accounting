mod bill_form_template;
mod bill_form_values;
mod bill_row;
mod catalog_sku_ref;
mod category_option;
mod expense_form_template;
mod expense_form_values;
mod expense_row;
mod index_template;
mod integration_form_template;
mod integration_form_values;
mod integration_row;
mod money_summary;
mod receipt_row;
pub(crate) use bill_form_template::BillFormTemplate;
pub use bill_form_values::BillFormValues;
pub use bill_row::BillRow;
pub use catalog_sku_ref::CatalogSkuRef;
pub(crate) use category_option::CategoryOption;
pub(crate) use expense_form_template::ExpenseFormTemplate;
pub use expense_form_values::ExpenseFormValues;
pub use expense_row::ExpenseRow;
pub(crate) use index_template::IndexTemplate;
pub(crate) use integration_form_template::IntegrationFormTemplate;
pub use integration_form_values::IntegrationFormValues;
pub use integration_row::IntegrationRow;
pub(crate) use money_summary::MoneySummary;
pub use receipt_row::ReceiptRow;

use askama::Template;
use chrono::{DateTime, Utc};

use crate::catalog::CatalogSku;
use crate::model::{
    Bill, BillKind, BillStatus, DATE_FORMAT, Expense, ExpenseCategory, Integration,
    IntegrationProvider, Receipt, format_line_items_text,
};
use sigma_theme::copyright_years;
use sigma_theme::nav::{SiteHeader, site_menu};
use sigma_theme::site_nav::{AppSiteNav, render_app_site_nav};

/// Timestamp format for table cells (minute precision, always UTC).
const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M UTC";

fn page_header() -> SiteHeader {
    SiteHeader::new("Accounting").with_menu(site_menu(None))
}

fn site_nav(return_path: &str) -> Result<String, askama::Error> {
    render_app_site_nav(&AppSiteNav {
        identity_base: &crate::config::identity_public_base_url(),
        app_base: &crate::config::public_base_url(),
        contact_base: &crate::config::contact_public_base_url(),
        cart_url: &crate::config::cart_public_base_url(),
        cart_count: 0,
        return_path,
        show_cart: true,
        show_contact_us: false,
        leading_html: "",
    })
}

fn timestamp_display(at: DateTime<Utc>) -> String {
    at.format(TIMESTAMP_FORMAT).to_string()
}

/// Admin link to a linked order, when both a public orders URL and an order
/// id are present.
fn order_href(orders_public_base: Option<&String>, order_id: Option<&String>) -> Option<String> {
    match (orders_public_base, order_id) {
        (Some(base), Some(order_id)) => Some(format!("{base}admin/orders/{order_id}")),
        _ => None,
    }
}

fn bill_rows(bills: Vec<Bill>) -> Vec<BillRow> {
    let orders_public_base = crate::config::orders_public_base_url();
    bills
        .into_iter()
        .map(|bill| BillRow {
            kind_label: bill.kind.label(),
            status_label: bill.status.label(),
            total_display: format_amount(bill.total_cents, &bill.currency),
            updated_display: timestamp_display(bill.updated_at),
            order_href: order_href(orders_public_base.as_ref(), bill.order_id.as_ref()),
            bill,
        })
        .collect()
}

fn category_options(selected: &str) -> Vec<CategoryOption> {
    ExpenseCategory::ALL
        .into_iter()
        .map(|category| CategoryOption {
            value: category.as_str(),
            label: category.label(),
            selected: category.as_str() == selected,
        })
        .collect()
}

fn expense_rows(expenses: Vec<Expense>) -> Vec<ExpenseRow> {
    let orders_public_base = crate::config::orders_public_base_url();
    expenses
        .into_iter()
        .map(|expense| ExpenseRow {
            category_label: expense.category.label(),
            amount_display: format_amount(expense.amount_cents, &expense.currency),
            order_href: order_href(orders_public_base.as_ref(), expense.order_id.as_ref()),
            expense,
        })
        .collect()
}

fn receipt_rows(receipts: Vec<Receipt>) -> Vec<ReceiptRow> {
    let orders_public_base = crate::config::orders_public_base_url();
    receipts
        .into_iter()
        .map(|receipt| ReceiptRow {
            kind_label: receipt.kind.label(),
            amount_display: format_amount(receipt.amount_cents, &receipt.currency),
            occurred_display: timestamp_display(receipt.occurred_at),
            order_href: order_href(orders_public_base.as_ref(), receipt.order_id.as_ref()),
            receipt,
        })
        .collect()
}

/// Per-currency money in / money out / net.
///
/// Money in is receipts with refunds subtracting; money out is expenses plus
/// bills already marked paid — an unpaid bill is a commitment, not cash that
/// has left the account.
fn money_summary(
    receipts: &[ReceiptRow],
    expenses: &[ExpenseRow],
    bills: &[BillRow],
) -> MoneySummary {
    let mut money_in: Vec<(String, i64)> = Vec::new();
    for row in receipts {
        let amount = row.receipt.kind.sign() * row.receipt.amount_cents;
        add_currency_total(&mut money_in, &row.receipt.currency, amount);
    }

    let mut money_out: Vec<(String, i64)> = Vec::new();
    for row in expenses {
        add_currency_total(
            &mut money_out,
            &row.expense.currency,
            row.expense.amount_cents,
        );
    }
    for row in bills.iter().filter(|r| r.bill.status == BillStatus::Paid) {
        add_currency_total(&mut money_out, &row.bill.currency, row.bill.total_cents);
    }

    let mut net = money_in.clone();
    for (currency, total) in &money_out {
        add_currency_total(&mut net, currency, -total);
    }

    MoneySummary {
        money_in: format_currency_totals(&money_in),
        money_out: format_currency_totals(&money_out),
        net: format_currency_totals(&net),
    }
}

/// Sum of listed expenses per currency, e.g. `USD 12.50 + EUR 3.00`.
fn expense_total_display(expenses: &[ExpenseRow]) -> Option<String> {
    let mut totals: Vec<(String, i64)> = Vec::new();
    for row in expenses {
        add_currency_total(&mut totals, &row.expense.currency, row.expense.amount_cents);
    }
    format_currency_totals(&totals)
}

/// Accumulate `amount` into the per-currency running totals.
fn add_currency_total(totals: &mut Vec<(String, i64)>, currency: &str, amount: i64) {
    match totals.iter_mut().find(|(c, _)| c == currency) {
        Some((_, total)) => *total += amount,
        None => totals.push((currency.to_string(), amount)),
    }
}

fn format_currency_totals(totals: &[(String, i64)]) -> Option<String> {
    if totals.is_empty() {
        return None;
    }
    Some(
        totals
            .iter()
            .map(|(currency, total)| format_amount(*total, currency))
            .collect::<Vec<_>>()
            .join(" + "),
    )
}

fn integration_rows(integrations: Vec<Integration>) -> Vec<IntegrationRow> {
    integrations
        .into_iter()
        .map(|integration| IntegrationRow {
            provider_label: integration.provider.label(),
            updated_display: timestamp_display(integration.updated_at),
            integration,
        })
        .collect()
}

fn catalog_sku_refs(skus: &[CatalogSku]) -> Vec<CatalogSkuRef> {
    skus.iter()
        .map(|sku| CatalogSkuRef {
            id: sku.id.clone(),
            sku_code: sku.sku_code.clone(),
            name: sku.name.clone(),
        })
        .collect()
}

/// `USD 12.50` — exact integer-cent formatting (no float rounding).
fn format_amount(cents: i64, currency: &str) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let cents = cents.unsigned_abs();
    format!("{currency} {sign}{}.{:02}", cents / 100, cents % 100)
}

fn values_from_bill(bill: &Bill) -> BillFormValues {
    BillFormValues {
        kind: bill.kind.as_str().to_string(),
        status: bill.status.as_str().to_string(),
        vendor: bill.vendor.clone(),
        invoice_number: bill.invoice_number.clone().unwrap_or_default(),
        order_id: bill.order_id.clone().unwrap_or_default(),
        bill_date: bill.bill_date.format(DATE_FORMAT).to_string(),
        due_date: bill
            .due_date
            .map(|date| date.format(DATE_FORMAT).to_string())
            .unwrap_or_default(),
        currency: bill.currency.clone(),
        line_items: format_line_items_text(&bill.line_items),
        scan_uri: bill.scan_uri.clone().unwrap_or_default(),
        notes: bill.notes.clone().unwrap_or_default(),
    }
}

fn default_bill_form_values() -> BillFormValues {
    BillFormValues {
        kind: BillKind::Digital.as_str().to_string(),
        status: BillStatus::Draft.as_str().to_string(),
        vendor: String::new(),
        invoice_number: String::new(),
        order_id: String::new(),
        bill_date: String::new(),
        due_date: String::new(),
        currency: "USD".to_string(),
        line_items: String::new(),
        scan_uri: String::new(),
        notes: String::new(),
    }
}

fn values_from_expense(expense: &Expense) -> ExpenseFormValues {
    ExpenseFormValues {
        expense_date: expense.expense_date.format(DATE_FORMAT).to_string(),
        category: expense.category.as_str().to_string(),
        description: expense.description.clone(),
        vendor: expense.vendor.clone().unwrap_or_default(),
        amount_cents: expense.amount_cents.to_string(),
        currency: expense.currency.clone(),
        receipt_uri: expense.receipt_uri.clone().unwrap_or_default(),
        bill_id: expense.bill_id.clone().unwrap_or_default(),
        order_id: expense.order_id.clone().unwrap_or_default(),
        notes: expense.notes.clone().unwrap_or_default(),
    }
}

fn default_expense_form_values() -> ExpenseFormValues {
    ExpenseFormValues {
        expense_date: String::new(),
        category: ExpenseCategory::Other.as_str().to_string(),
        description: String::new(),
        vendor: String::new(),
        amount_cents: String::new(),
        currency: "USD".to_string(),
        receipt_uri: String::new(),
        bill_id: String::new(),
        order_id: String::new(),
        notes: String::new(),
    }
}

fn values_from_integration(integration: &Integration) -> IntegrationFormValues {
    IntegrationFormValues {
        name: integration.name.clone(),
        provider: integration.provider.as_str().to_string(),
        enabled: integration.enabled,
        external_account_id: integration.external_account_id.clone().unwrap_or_default(),
        webhook_url: integration.webhook_url.clone().unwrap_or_default(),
        notes: integration.notes.clone().unwrap_or_default(),
    }
}

fn default_integration_form_values() -> IntegrationFormValues {
    IntegrationFormValues {
        name: String::new(),
        provider: IntegrationProvider::QuickBooks.as_str().to_string(),
        enabled: true,
        external_account_id: String::new(),
        webhook_url: String::new(),
        notes: String::new(),
    }
}

fn render_bill_form(
    catalog_skus: &[CatalogSku],
    bill: Option<Bill>,
    error: Option<String>,
    values: BillFormValues,
) -> Result<String, askama::Error> {
    let kind = values.kind.to_lowercase();
    let status = values.status.to_lowercase();
    let return_path = bill
        .as_ref()
        .map(|entry| format!("/bills/{}/edit", entry.id))
        .unwrap_or_else(|| "/bills/new".to_string());
    BillFormTemplate {
        bill,
        kind_scanned: kind == BillKind::Scanned.as_str(),
        kind_digital: kind == BillKind::Digital.as_str(),
        status_draft: status == BillStatus::Draft.as_str(),
        status_approved: status == BillStatus::Approved.as_str(),
        status_paid: status == BillStatus::Paid.as_str(),
        status_void: status == BillStatus::Void.as_str(),
        vendor: values.vendor,
        invoice_number: values.invoice_number,
        order_id: values.order_id,
        bill_date: values.bill_date,
        due_date: values.due_date,
        currency: values.currency,
        line_items: values.line_items,
        scan_uri: values.scan_uri,
        notes: values.notes,
        catalog_skus: catalog_sku_refs(catalog_skus),
        error,
        site_header: page_header(),
        site_nav: site_nav(&return_path)?,
        copyright_years: copyright_years(),
    }
    .render()
}

fn render_expense_form(
    expense: Option<Expense>,
    error: Option<String>,
    values: ExpenseFormValues,
) -> Result<String, askama::Error> {
    let category = values.category.to_lowercase();
    let return_path = expense
        .as_ref()
        .map(|entry| format!("/expenses/{}/edit", entry.id))
        .unwrap_or_else(|| "/expenses/new".to_string());
    ExpenseFormTemplate {
        expense,
        expense_date: values.expense_date,
        category_options: category_options(&category),
        description: values.description,
        vendor: values.vendor,
        amount_cents: values.amount_cents,
        currency: values.currency,
        receipt_uri: values.receipt_uri,
        bill_id: values.bill_id,
        order_id: values.order_id,
        notes: values.notes,
        error,
        site_header: page_header(),
        site_nav: site_nav(&return_path)?,
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
    let return_path = integration
        .as_ref()
        .map(|entry| format!("/integrations/{}/edit", entry.id))
        .unwrap_or_else(|| "/integrations/new".to_string());
    IntegrationFormTemplate {
        integration,
        name: values.name,
        provider_quickbooks: provider == IntegrationProvider::QuickBooks.as_str(),
        provider_xero: provider == IntegrationProvider::Xero.as_str(),
        provider_custom: provider == IntegrationProvider::Custom.as_str(),
        enabled: values.enabled,
        external_account_id: values.external_account_id,
        webhook_url: values.webhook_url,
        notes: values.notes,
        error,
        site_header: page_header(),
        site_nav: site_nav(&return_path)?,
        copyright_years: copyright_years(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_index_html(
    bills: Vec<Bill>,
    expenses: Vec<Expense>,
    receipts: Vec<Receipt>,
    integrations: Vec<Integration>,
    catalog_skus: &[CatalogSku],
    catalog_notice: Option<String>,
    message: Option<String>,
) -> Result<String, askama::Error> {
    let bills = bill_rows(bills);
    let expenses = expense_rows(expenses);
    let receipts = receipt_rows(receipts);
    let expense_total = expense_total_display(&expenses);
    let money = money_summary(&receipts, &expenses, &bills);
    IndexTemplate {
        bills,
        expenses,
        expense_total,
        receipts,
        money,
        integrations: integration_rows(integrations),
        catalog_skus: catalog_sku_refs(catalog_skus),
        catalog_notice,
        catalog_configured: crate::config::catalog_configured(),
        payments_configured: crate::config::payments_configured(),
        message,
        site_header: page_header(),
        site_nav: site_nav("/")?,
        copyright_years: copyright_years(),
    }
    .render()
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_bill_form_html(
    catalog_skus: &[CatalogSku],
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
    catalog_skus: &[CatalogSku],
    bill: Option<Bill>,
    error: Option<String>,
    values: BillFormValues,
) -> Result<String, askama::Error> {
    render_bill_form(catalog_skus, bill, error, values)
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_expense_form_html(
    expense: Option<Expense>,
    error: Option<String>,
) -> Result<String, askama::Error> {
    let values = expense
        .as_ref()
        .map(values_from_expense)
        .unwrap_or_else(default_expense_form_values);
    render_expense_form(expense, error, values)
}

/// # Errors
///
/// Returns [`askama::Error`] when template rendering fails.
pub fn render_expense_form_html_with_values(
    expense: Option<Expense>,
    error: Option<String>,
    values: ExpenseFormValues,
) -> Result<String, askama::Error> {
    render_expense_form(expense, error, values)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_amount_uses_exact_integer_cents() {
        assert_eq!(format_amount(0, "USD"), "USD 0.00");
        assert_eq!(format_amount(5, "USD"), "USD 0.05");
        assert_eq!(format_amount(1250, "USD"), "USD 12.50");
        assert_eq!(format_amount(-1250, "USD"), "USD -12.50");
        assert_eq!(format_amount(i64::MAX, "USD"), "USD 92233720368547758.07");
    }

    #[test]
    fn category_options_mark_the_selected_value() {
        let options = category_options("tooling");
        assert_eq!(options.len(), ExpenseCategory::ALL.len());
        let selected: Vec<&str> = options
            .iter()
            .filter(|option| option.selected)
            .map(|option| option.value)
            .collect();
        assert_eq!(selected, vec!["tooling"]);
    }
}
