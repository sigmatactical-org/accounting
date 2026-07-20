mod bill;
mod bill_form;
mod bill_kind;
mod bill_line_item;
mod bill_status;
mod create_bill;
mod create_expense;
mod create_integration;
mod create_receipt;
mod expense;
mod expense_category;
mod expense_form;
mod integration;
mod integration_form;
mod integration_provider;
mod receipt;
mod receipt_kind;
mod update_bill;
mod update_expense;
mod update_integration;
pub use bill::Bill;
pub use bill_form::BillForm;
pub use bill_kind::BillKind;
pub use bill_line_item::BillLineItem;
pub use bill_status::BillStatus;
pub use create_bill::CreateBill;
pub use create_expense::CreateExpense;
pub use create_integration::CreateIntegration;
pub use create_receipt::CreateReceipt;
pub use expense::Expense;
pub use expense_category::ExpenseCategory;
pub use expense_form::ExpenseForm;
pub use integration::Integration;
pub use integration_form::IntegrationForm;
pub use integration_provider::IntegrationProvider;
pub use receipt::Receipt;
pub use receipt_kind::ReceiptKind;
pub use update_bill::UpdateBill;
pub use update_expense::UpdateExpense;
pub use update_integration::UpdateIntegration;

use chrono::NaiveDate;

/// Date format used by HTML `<input type="date">` and the JSON API.
pub(crate) const DATE_FORMAT: &str = "%Y-%m-%d";

fn default_currency() -> String {
    "USD".to_string()
}

fn normalize_currency(value: String) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        default_currency()
    } else {
        trimmed.to_uppercase()
    }
}

/// Parse a `YYYY-MM-DD` form field, naming the field in the error message.
pub(crate) fn parse_date(value: &str, field: &str) -> Result<NaiveDate, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{field} is required"));
    }
    NaiveDate::parse_from_str(value, DATE_FORMAT).map_err(|e| format!("invalid {field}: {e}"))
}

/// Parse an optional `YYYY-MM-DD` form field (blank yields `None`).
pub(crate) fn parse_optional_date(value: &str, field: &str) -> Result<Option<NaiveDate>, String> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    parse_date(value, field).map(Some)
}

/// Parse line items as `<sku_id|-> description qty unit_cents` (whitespace-separated).
pub fn parse_line_items_text(text: &str) -> Result<Vec<BillLineItem>, String> {
    let mut items = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let sku_token = parts
            .next()
            .ok_or_else(|| format!("line {}: missing sku id or '-'", line_no + 1))?;
        let description = parts
            .next()
            .ok_or_else(|| format!("line {}: missing description", line_no + 1))?;
        let qty_str = parts
            .next()
            .ok_or_else(|| format!("line {}: missing quantity", line_no + 1))?;
        let price_str = parts
            .next()
            .ok_or_else(|| format!("line {}: missing unit price (cents)", line_no + 1))?;
        if parts.next().is_some() {
            return Err(format!("line {}: too many fields", line_no + 1));
        }
        let quantity: u32 = qty_str
            .parse()
            .map_err(|_| format!("line {}: invalid quantity", line_no + 1))?;
        if quantity == 0 {
            return Err(format!("line {}: quantity must be at least 1", line_no + 1));
        }
        let unit_price_cents: i64 = price_str
            .parse()
            .map_err(|_| format!("line {}: invalid unit price (cents)", line_no + 1))?;
        let sku_id = if sku_token == "-" {
            None
        } else {
            Some(sku_token.to_string())
        };
        items.push(BillLineItem {
            sku_id,
            description: description.to_string(),
            quantity,
            unit_price_cents,
        });
    }
    Ok(items)
}

/// Render line items as the multi-line `<sku_id|-> description qty unit_cents`
/// text form parsed by [`parse_line_items_text`].
pub fn format_line_items_text(items: &[BillLineItem]) -> String {
    items
        .iter()
        .map(|item| {
            let sku = item.sku_id.as_deref().unwrap_or("-");
            format!(
                "{sku} {} {} {}",
                item.description, item.quantity, item.unit_price_cents
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Sum of line-item totals in cents.
pub fn compute_total_cents(items: &[BillLineItem]) -> i64 {
    items
        .iter()
        .map(|item| item.quantity as i64 * item.unit_price_cents)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_items_skips_comments_and_blank_lines() {
        let text = "# header\n\nabc123 Widget 2 1500\n- Shipping 1 500\n";
        let items = parse_line_items_text(text).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].sku_id.as_deref(), Some("abc123"));
        assert_eq!(items[0].description, "Widget");
        assert_eq!(items[0].quantity, 2);
        assert_eq!(items[0].unit_price_cents, 1500);
        assert!(items[1].sku_id.is_none());
    }

    #[test]
    fn compute_total_cents_sums_line_items() {
        let items = vec![
            BillLineItem {
                sku_id: None,
                description: "A".to_string(),
                quantity: 2,
                unit_price_cents: 100,
            },
            BillLineItem {
                sku_id: None,
                description: "B".to_string(),
                quantity: 1,
                unit_price_cents: 50,
            },
        ];
        assert_eq!(compute_total_cents(&items), 250);
    }

    #[test]
    fn parse_date_reports_missing_and_malformed_input() {
        assert_eq!(
            parse_date("2026-01-15", "bill date"),
            Ok(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap())
        );
        assert_eq!(
            parse_date("  ", "bill date"),
            Err("bill date is required".to_string())
        );
        assert!(parse_date("nope", "bill date").is_err());
        assert_eq!(parse_optional_date("", "due date"), Ok(None));
    }

    #[test]
    fn enums_round_trip_through_as_str() {
        for kind in BillKind::ALL {
            assert_eq!(kind.as_str().parse::<BillKind>(), Ok(kind));
        }
        for status in BillStatus::ALL {
            assert_eq!(status.as_str().parse::<BillStatus>(), Ok(status));
        }
        for category in ExpenseCategory::ALL {
            assert_eq!(category.as_str().parse::<ExpenseCategory>(), Ok(category));
        }
        for provider in IntegrationProvider::ALL {
            assert_eq!(
                provider.as_str().parse::<IntegrationProvider>(),
                Ok(provider)
            );
        }
        assert!("nope".parse::<BillKind>().is_err());
    }
}
