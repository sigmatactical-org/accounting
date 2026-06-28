use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BillKind {
    Scanned,
    Digital,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BillStatus {
    Draft,
    Approved,
    Paid,
    Void,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BillLineItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku_id: Option<String>,
    pub description: String,
    pub quantity: u32,
    pub unit_price_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bill {
    pub id: String,
    pub kind: BillKind,
    pub status: BillStatus,
    pub vendor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,
    pub bill_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub line_items: Vec<BillLineItem>,
    pub total_cents: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub updated_at: String,
}

fn default_currency() -> String {
    "USD".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBill {
    pub kind: BillKind,
    #[serde(default)]
    pub status: Option<BillStatus>,
    pub vendor: String,
    pub invoice_number: Option<String>,
    pub bill_date: String,
    pub due_date: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub line_items: Vec<BillLineItem>,
    pub scan_uri: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBill {
    pub kind: BillKind,
    pub status: BillStatus,
    pub vendor: String,
    pub invoice_number: Option<String>,
    pub bill_date: String,
    pub due_date: Option<String>,
    pub currency: String,
    #[serde(default)]
    pub line_items: Vec<BillLineItem>,
    pub scan_uri: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IntegrationProvider {
    QuickBooks,
    Xero,
    Custom,
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct CreateIntegration {
    pub name: String,
    pub provider: IntegrationProvider,
    #[serde(default)]
    pub enabled: Option<bool>,
    pub external_account_id: Option<String>,
    pub webhook_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateIntegration {
    pub name: String,
    pub provider: IntegrationProvider,
    pub enabled: bool,
    pub external_account_id: Option<String>,
    pub webhook_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BillForm {
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

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationForm {
    pub name: String,
    pub provider: String,
    pub enabled: Option<String>,
    pub external_account_id: String,
    pub webhook_url: String,
    pub notes: String,
}

impl BillForm {
    pub fn into_create(self) -> Result<CreateBill, String> {
        Ok(CreateBill {
            kind: parse_bill_kind(&self.kind)?,
            status: Some(parse_bill_status(&self.status)?),
            vendor: self.vendor,
            invoice_number: empty_to_none(self.invoice_number),
            bill_date: self.bill_date,
            due_date: empty_to_none(self.due_date),
            currency: empty_to_none(self.currency),
            line_items: parse_line_items_text(&self.line_items)?,
            scan_uri: empty_to_none(self.scan_uri),
            notes: empty_to_none(self.notes),
        })
    }

    pub fn into_update(self) -> Result<UpdateBill, String> {
        Ok(UpdateBill {
            kind: parse_bill_kind(&self.kind)?,
            status: parse_bill_status(&self.status)?,
            vendor: self.vendor,
            invoice_number: empty_to_none(self.invoice_number),
            bill_date: self.bill_date,
            due_date: empty_to_none(self.due_date),
            currency: normalize_currency(self.currency),
            line_items: parse_line_items_text(&self.line_items)?,
            scan_uri: empty_to_none(self.scan_uri),
            notes: empty_to_none(self.notes),
        })
    }
}

impl IntegrationForm {
    pub fn into_create(self) -> Result<CreateIntegration, String> {
        Ok(CreateIntegration {
            name: self.name,
            provider: parse_integration_provider(&self.provider)?,
            enabled: Some(self.enabled.is_some()),
            external_account_id: empty_to_none(self.external_account_id),
            webhook_url: empty_to_none(self.webhook_url),
            notes: empty_to_none(self.notes),
        })
    }

    pub fn into_update(self) -> Result<UpdateIntegration, String> {
        Ok(UpdateIntegration {
            name: self.name,
            provider: parse_integration_provider(&self.provider)?,
            enabled: self.enabled.is_some(),
            external_account_id: empty_to_none(self.external_account_id),
            webhook_url: empty_to_none(self.webhook_url),
            notes: empty_to_none(self.notes),
        })
    }
}

fn empty_to_none(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_currency(value: String) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        default_currency()
    } else {
        trimmed.to_uppercase()
    }
}

fn parse_bill_kind(value: &str) -> Result<BillKind, String> {
    match value.trim().to_lowercase().as_str() {
        "scanned" => Ok(BillKind::Scanned),
        "digital" => Ok(BillKind::Digital),
        other => Err(format!("invalid bill kind: {other}")),
    }
}

fn parse_bill_status(value: &str) -> Result<BillStatus, String> {
    match value.trim().to_lowercase().as_str() {
        "draft" => Ok(BillStatus::Draft),
        "approved" => Ok(BillStatus::Approved),
        "paid" => Ok(BillStatus::Paid),
        "void" => Ok(BillStatus::Void),
        other => Err(format!("invalid bill status: {other}")),
    }
}

fn parse_integration_provider(value: &str) -> Result<IntegrationProvider, String> {
    match value.trim().to_lowercase().as_str() {
        "quickbooks" => Ok(IntegrationProvider::QuickBooks),
        "xero" => Ok(IntegrationProvider::Xero),
        "custom" => Ok(IntegrationProvider::Custom),
        other => Err(format!("invalid integration provider: {other}")),
    }
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

pub fn compute_total_cents(items: &[BillLineItem]) -> i64 {
    items
        .iter()
        .map(|item| item.quantity as i64 * item.unit_price_cents)
        .sum()
}

impl Bill {
    pub fn new(input: CreateBill) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let total_cents = compute_total_cents(&input.line_items);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            kind: input.kind,
            status: input.status.unwrap_or(BillStatus::Draft),
            vendor: input.vendor.trim().to_string(),
            invoice_number: input.invoice_number.map(|s| s.trim().to_string()),
            bill_date: input.bill_date.trim().to_string(),
            due_date: input.due_date.map(|s| s.trim().to_string()),
            currency: input
                .currency
                .map(|s| s.trim().to_uppercase())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(default_currency),
            line_items: input.line_items,
            total_cents,
            scan_uri: input.scan_uri.map(|s| s.trim().to_string()),
            notes: input.notes.map(|s| s.trim().to_string()),
            updated_at: now,
        }
    }

    pub fn apply_update(&mut self, input: UpdateBill) {
        self.kind = input.kind;
        self.status = input.status;
        self.vendor = input.vendor.trim().to_string();
        self.invoice_number = input.invoice_number.map(|s| s.trim().to_string());
        self.bill_date = input.bill_date.trim().to_string();
        self.due_date = input.due_date.map(|s| s.trim().to_string());
        self.currency = normalize_currency(input.currency);
        self.line_items = input.line_items;
        self.total_cents = compute_total_cents(&self.line_items);
        self.scan_uri = input.scan_uri.map(|s| s.trim().to_string());
        self.notes = input.notes.map(|s| s.trim().to_string());
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

impl Integration {
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
}
