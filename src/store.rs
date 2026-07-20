mod store_error;
pub use store_error::{StoreError, store_error_status};

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, Row};

use crate::model::{
    Bill, BillKind, BillLineItem, BillStatus, CreateBill, CreateExpense, CreateIntegration,
    CreateReceipt, Expense, ExpenseCategory, Integration, IntegrationProvider, Receipt,
    ReceiptKind, UpdateBill, UpdateExpense, UpdateIntegration,
};

#[derive(Debug, Clone)]
pub struct AccountingStore {
    pool: PgPool,
}

impl AccountingStore {
    pub async fn connect() -> Result<Self, StoreError> {
        let pool = sigma_pg::connect_as("accounting").await?;
        Ok(Self { pool })
    }

    #[cfg(test)]
    pub async fn connect_empty() -> Result<Self, StoreError> {
        let store = Self::connect().await?;
        sigma_pg::assert_disposable_test_db(&store.pool).await;
        sqlx::query(
            "TRUNCATE accounting.receipts, accounting.expenses, accounting.bill_line_items, \
             accounting.bills, accounting.integrations",
        )
        .execute(&store.pool)
        .await?;
        Ok(store)
    }

    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn list_bills(&self) -> Result<Vec<Bill>, StoreError> {
        let rows = sqlx::query(
            "SELECT id, kind, status, vendor, invoice_number, order_id, bill_date, due_date, \
             currency, total_cents, scan_uri, notes, updated_at \
             FROM accounting.bills ORDER BY bill_date DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        self.rows_to_bills(rows).await
    }

    pub async fn get_bill(&self, id: &str) -> Result<Option<Bill>, StoreError> {
        let row = sqlx::query(
            "SELECT id, kind, status, vendor, invoice_number, order_id, bill_date, due_date, \
             currency, total_cents, scan_uri, notes, updated_at \
             FROM accounting.bills WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => Ok(self.rows_to_bills(vec![row]).await?.into_iter().next()),
            None => Ok(None),
        }
    }

    pub async fn create_bill(&self, input: CreateBill) -> Result<Bill, StoreError> {
        validate_bill_input(
            input.kind,
            &input.vendor,
            &input.line_items,
            input.scan_uri.as_deref(),
        )?;
        let bill = Bill::new(input);
        let mut tx = self.pool.begin().await?;
        insert_bill(&mut tx, &bill).await?;
        replace_line_items(&mut tx, &bill.id, &bill.line_items).await?;
        tx.commit().await?;
        Ok(bill)
    }

    pub async fn update_bill(&self, id: &str, input: UpdateBill) -> Result<Bill, StoreError> {
        let mut bill = self.get_bill(id).await?.ok_or(StoreError::BillNotFound)?;
        validate_bill_input(
            input.kind,
            &input.vendor,
            &input.line_items,
            input.scan_uri.as_deref(),
        )?;
        bill.apply_update(input);
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "UPDATE accounting.bills SET kind = $2, status = $3, vendor = $4, invoice_number = $5, \
             order_id = $6, bill_date = $7, due_date = $8, currency = $9, total_cents = $10, \
             scan_uri = $11, notes = $12, updated_at = $13 WHERE id = $1",
        )
        .bind(&bill.id)
        .bind(bill.kind.as_str())
        .bind(bill.status.as_str())
        .bind(&bill.vendor)
        .bind(&bill.invoice_number)
        .bind(&bill.order_id)
        .bind(bill.bill_date)
        .bind(bill.due_date)
        .bind(&bill.currency)
        .bind(bill.total_cents)
        .bind(&bill.scan_uri)
        .bind(&bill.notes)
        .bind(bill.updated_at)
        .execute(&mut *tx)
        .await?;
        replace_line_items(&mut tx, &bill.id, &bill.line_items).await?;
        tx.commit().await?;
        Ok(bill)
    }

    pub async fn delete_bill(&self, id: &str) -> Result<(), StoreError> {
        self.delete_by_id(
            "DELETE FROM accounting.bills WHERE id = $1",
            id,
            StoreError::BillNotFound,
        )
        .await
    }

    pub async fn list_expenses(&self) -> Result<Vec<Expense>, StoreError> {
        let rows = sqlx::query(
            "SELECT id, expense_date, category, description, vendor, amount_cents, currency, \
             receipt_uri, bill_id, order_id, notes, updated_at \
             FROM accounting.expenses ORDER BY expense_date DESC, id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_expense).collect()
    }

    pub async fn get_expense(&self, id: &str) -> Result<Option<Expense>, StoreError> {
        let row = sqlx::query(
            "SELECT id, expense_date, category, description, vendor, amount_cents, currency, \
             receipt_uri, bill_id, order_id, notes, updated_at \
             FROM accounting.expenses WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_expense).transpose()
    }

    pub async fn create_expense(&self, input: CreateExpense) -> Result<Expense, StoreError> {
        self.validate_expense_input(
            &input.description,
            input.amount_cents,
            input.bill_id.as_deref(),
        )
        .await?;
        let expense = Expense::new(input);
        sqlx::query(
            "INSERT INTO accounting.expenses \
             (id, expense_date, category, description, vendor, amount_cents, currency, \
              receipt_uri, bill_id, order_id, notes, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(&expense.id)
        .bind(expense.expense_date)
        .bind(expense.category.as_str())
        .bind(&expense.description)
        .bind(&expense.vendor)
        .bind(expense.amount_cents)
        .bind(&expense.currency)
        .bind(&expense.receipt_uri)
        .bind(&expense.bill_id)
        .bind(&expense.order_id)
        .bind(&expense.notes)
        .bind(expense.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(expense)
    }

    pub async fn update_expense(
        &self,
        id: &str,
        input: UpdateExpense,
    ) -> Result<Expense, StoreError> {
        let mut expense = self
            .get_expense(id)
            .await?
            .ok_or(StoreError::ExpenseNotFound)?;
        self.validate_expense_input(
            &input.description,
            input.amount_cents,
            input.bill_id.as_deref(),
        )
        .await?;
        expense.apply_update(input);
        sqlx::query(
            "UPDATE accounting.expenses SET expense_date = $2, category = $3, description = $4, \
             vendor = $5, amount_cents = $6, currency = $7, receipt_uri = $8, bill_id = $9, \
             order_id = $10, notes = $11, updated_at = $12 WHERE id = $1",
        )
        .bind(&expense.id)
        .bind(expense.expense_date)
        .bind(expense.category.as_str())
        .bind(&expense.description)
        .bind(&expense.vendor)
        .bind(expense.amount_cents)
        .bind(&expense.currency)
        .bind(&expense.receipt_uri)
        .bind(&expense.bill_id)
        .bind(&expense.order_id)
        .bind(&expense.notes)
        .bind(expense.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(expense)
    }

    pub async fn delete_expense(&self, id: &str) -> Result<(), StoreError> {
        self.delete_by_id(
            "DELETE FROM accounting.expenses WHERE id = $1",
            id,
            StoreError::ExpenseNotFound,
        )
        .await
    }

    pub async fn list_receipts(&self) -> Result<Vec<Receipt>, StoreError> {
        let rows = sqlx::query(
            "SELECT id, charge_id, order_id, user_id, kind, amount_cents, currency, occurred_at, \
             notes, updated_at FROM accounting.receipts ORDER BY occurred_at DESC, id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_receipt).collect()
    }

    pub async fn get_receipt(&self, id: &str) -> Result<Option<Receipt>, StoreError> {
        self.receipt_by(
            "SELECT id, charge_id, order_id, user_id, kind, amount_cents, currency, occurred_at, \
             notes, updated_at FROM accounting.receipts WHERE id = $1",
            id,
        )
        .await
    }

    pub async fn get_receipt_by_charge(
        &self,
        charge_id: &str,
    ) -> Result<Option<Receipt>, StoreError> {
        self.receipt_by(
            "SELECT id, charge_id, order_id, user_id, kind, amount_cents, currency, occurred_at, \
             notes, updated_at FROM accounting.receipts WHERE charge_id = $1",
            charge_id,
        )
        .await
    }

    /// Fetch at most one receipt by a unique column.
    async fn receipt_by(
        &self,
        statement: &'static str,
        value: &str,
    ) -> Result<Option<Receipt>, StoreError> {
        let row = sqlx::query(statement)
            .bind(value)
            .fetch_optional(&self.pool)
            .await?;
        row.map(row_to_receipt).transpose()
    }

    /// Record money received. Idempotent on `charge_id`: recording a charge
    /// that already has a receipt returns the existing row with `false`, so
    /// the cart's checkout push and the reconcile sweep can safely overlap.
    pub async fn record_receipt(
        &self,
        input: CreateReceipt,
    ) -> Result<(Receipt, bool), StoreError> {
        if input.charge_id.trim().is_empty() {
            return Err(StoreError::ReceiptChargeRequired);
        }
        if input.user_id.trim().is_empty() {
            return Err(StoreError::ReceiptUserRequired);
        }
        if input.amount_cents < 1 {
            return Err(StoreError::InvalidAmount);
        }
        let receipt = Receipt::new(input);
        let result = sqlx::query(
            "INSERT INTO accounting.receipts \
             (id, charge_id, order_id, user_id, kind, amount_cents, currency, occurred_at, \
              notes, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (charge_id) DO NOTHING",
        )
        .bind(&receipt.id)
        .bind(&receipt.charge_id)
        .bind(&receipt.order_id)
        .bind(&receipt.user_id)
        .bind(receipt.kind.as_str())
        .bind(receipt.amount_cents)
        .bind(&receipt.currency)
        .bind(receipt.occurred_at)
        .bind(&receipt.notes)
        .bind(receipt.updated_at)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            let existing = self
                .get_receipt_by_charge(&receipt.charge_id)
                .await?
                .ok_or(StoreError::ReceiptNotFound)?;
            return Ok((existing, false));
        }
        Ok((receipt, true))
    }

    pub async fn delete_receipt(&self, id: &str) -> Result<(), StoreError> {
        self.delete_by_id(
            "DELETE FROM accounting.receipts WHERE id = $1",
            id,
            StoreError::ReceiptNotFound,
        )
        .await
    }

    async fn validate_expense_input(
        &self,
        description: &str,
        amount_cents: i64,
        bill_id: Option<&str>,
    ) -> Result<(), StoreError> {
        if description.trim().is_empty() {
            return Err(StoreError::ExpenseDescriptionRequired);
        }
        if amount_cents < 1 {
            return Err(StoreError::InvalidAmount);
        }
        if let Some(bill_id) = bill_id.map(str::trim).filter(|s| !s.is_empty()) {
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM accounting.bills WHERE id = $1)")
                    .bind(bill_id)
                    .fetch_one(&self.pool)
                    .await?;
            if !exists {
                return Err(StoreError::LinkedBillNotFound);
            }
        }
        Ok(())
    }

    pub async fn list_integrations(&self) -> Result<Vec<Integration>, StoreError> {
        let rows = sqlx::query(
            "SELECT id, name, provider, enabled, external_account_id, webhook_url, notes, \
             updated_at FROM accounting.integrations ORDER BY lower(name)",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_integration).collect()
    }

    pub async fn get_integration(&self, id: &str) -> Result<Option<Integration>, StoreError> {
        let row = sqlx::query(
            "SELECT id, name, provider, enabled, external_account_id, webhook_url, notes, \
             updated_at FROM accounting.integrations WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_integration).transpose()
    }

    pub async fn create_integration(
        &self,
        input: CreateIntegration,
    ) -> Result<Integration, StoreError> {
        self.validate_integration_name(&input.name, None).await?;
        let integration = Integration::new(input);
        sqlx::query(
            "INSERT INTO accounting.integrations \
             (id, name, provider, enabled, external_account_id, webhook_url, notes, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&integration.id)
        .bind(&integration.name)
        .bind(integration.provider.as_str())
        .bind(integration.enabled)
        .bind(&integration.external_account_id)
        .bind(&integration.webhook_url)
        .bind(&integration.notes)
        .bind(integration.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(integration)
    }

    pub async fn update_integration(
        &self,
        id: &str,
        input: UpdateIntegration,
    ) -> Result<Integration, StoreError> {
        let mut integration = self
            .get_integration(id)
            .await?
            .ok_or(StoreError::IntegrationNotFound)?;
        self.validate_integration_name(&input.name, Some(id))
            .await?;
        integration.apply_update(input);
        sqlx::query(
            "UPDATE accounting.integrations SET name = $2, provider = $3, enabled = $4, \
             external_account_id = $5, webhook_url = $6, notes = $7, updated_at = $8 \
             WHERE id = $1",
        )
        .bind(&integration.id)
        .bind(&integration.name)
        .bind(integration.provider.as_str())
        .bind(integration.enabled)
        .bind(&integration.external_account_id)
        .bind(&integration.webhook_url)
        .bind(&integration.notes)
        .bind(integration.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(integration)
    }

    pub async fn delete_integration(&self, id: &str) -> Result<(), StoreError> {
        self.delete_by_id(
            "DELETE FROM accounting.integrations WHERE id = $1",
            id,
            StoreError::IntegrationNotFound,
        )
        .await
    }

    /// Run a `DELETE ... WHERE id = $1`, reporting `missing` when nothing
    /// matched.
    async fn delete_by_id(
        &self,
        statement: &'static str,
        id: &str,
        missing: StoreError,
    ) -> Result<(), StoreError> {
        let result = sqlx::query(statement).bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(missing);
        }
        Ok(())
    }

    async fn rows_to_bills(
        &self,
        rows: Vec<sqlx::postgres::PgRow>,
    ) -> Result<Vec<Bill>, StoreError> {
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<String> = rows.iter().map(|r| r.get("id")).collect();
        let line_rows = sqlx::query(
            "SELECT bill_id, line_no, sku_id, description, quantity, unit_price_cents \
             FROM accounting.bill_line_items WHERE bill_id = ANY($1) ORDER BY bill_id, line_no",
        )
        .bind(&ids)
        .fetch_all(&self.pool)
        .await?;
        let mut line_items: HashMap<String, Vec<BillLineItem>> = HashMap::new();
        for row in line_rows {
            let bill_id: String = row.get("bill_id");
            line_items
                .entry(bill_id)
                .or_default()
                .push(row_to_line_item(row));
        }
        rows.into_iter()
            .map(|row| {
                let id: String = row.get("id");
                row_to_bill(row, line_items.remove(&id).unwrap_or_default())
            })
            .collect()
    }

    async fn validate_integration_name(
        &self,
        name: &str,
        except_id: Option<&str>,
    ) -> Result<(), StoreError> {
        if name.trim().is_empty() {
            return Err(StoreError::IntegrationNameRequired);
        }
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM accounting.integrations
                WHERE lower(name) = lower($1)
                  AND ($2::text IS NULL OR id <> $2)
             )",
        )
        .bind(name.trim())
        .bind(except_id)
        .fetch_one(&self.pool)
        .await?;
        if exists {
            return Err(StoreError::DuplicateIntegrationName);
        }
        Ok(())
    }
}

fn validate_bill_input(
    kind: BillKind,
    vendor: &str,
    line_items: &[BillLineItem],
    scan_uri: Option<&str>,
) -> Result<(), StoreError> {
    if vendor.trim().is_empty() {
        return Err(StoreError::VendorRequired);
    }
    if line_items.is_empty() {
        return Err(StoreError::BillNeedsLineItems);
    }
    if line_items.iter().any(|item| item.quantity == 0) {
        return Err(StoreError::InvalidQuantity);
    }
    if kind == BillKind::Scanned && scan_uri.map(str::trim).filter(|s| !s.is_empty()).is_none() {
        return Err(StoreError::ScanUriRequired);
    }
    Ok(())
}

async fn insert_bill(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    bill: &Bill,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO accounting.bills \
         (id, kind, status, vendor, invoice_number, order_id, bill_date, due_date, currency, \
          total_cents, scan_uri, notes, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
    )
    .bind(&bill.id)
    .bind(bill.kind.as_str())
    .bind(bill.status.as_str())
    .bind(&bill.vendor)
    .bind(&bill.invoice_number)
    .bind(&bill.order_id)
    .bind(bill.bill_date)
    .bind(bill.due_date)
    .bind(&bill.currency)
    .bind(bill.total_cents)
    .bind(&bill.scan_uri)
    .bind(&bill.notes)
    .bind(bill.updated_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn replace_line_items(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    bill_id: &str,
    items: &[BillLineItem],
) -> Result<(), StoreError> {
    sqlx::query("DELETE FROM accounting.bill_line_items WHERE bill_id = $1")
        .bind(bill_id)
        .execute(&mut **tx)
        .await?;
    for (line_no, item) in items.iter().enumerate() {
        sqlx::query(
            "INSERT INTO accounting.bill_line_items \
             (bill_id, line_no, sku_id, description, quantity, unit_price_cents) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(bill_id)
        .bind(line_no as i16)
        .bind(&item.sku_id)
        .bind(&item.description)
        .bind(item.quantity as i32)
        .bind(item.unit_price_cents)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// Parse an enum column, falling back to `default` for values written by a
/// schema newer than this build knows about.
fn enum_column<T: std::str::FromStr>(row: &sqlx::postgres::PgRow, column: &str, default: T) -> T {
    row.get::<String, _>(column).parse().unwrap_or(default)
}

fn row_to_bill(
    row: sqlx::postgres::PgRow,
    line_items: Vec<BillLineItem>,
) -> Result<Bill, StoreError> {
    Ok(Bill {
        kind: enum_column(&row, "kind", BillKind::Digital),
        status: enum_column(&row, "status", BillStatus::Draft),
        id: row.get("id"),
        vendor: row.get("vendor"),
        invoice_number: row.get("invoice_number"),
        order_id: row.get("order_id"),
        bill_date: row.get::<NaiveDate, _>("bill_date"),
        due_date: row.get::<Option<NaiveDate>, _>("due_date"),
        currency: row.get("currency"),
        line_items,
        total_cents: row.get("total_cents"),
        scan_uri: row.get("scan_uri"),
        notes: row.get("notes"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
}

fn row_to_line_item(row: sqlx::postgres::PgRow) -> BillLineItem {
    BillLineItem {
        sku_id: row.get("sku_id"),
        description: row.get("description"),
        quantity: row.get::<i32, _>("quantity") as u32,
        unit_price_cents: row.get("unit_price_cents"),
    }
}

fn row_to_integration(row: sqlx::postgres::PgRow) -> Result<Integration, StoreError> {
    Ok(Integration {
        provider: enum_column(&row, "provider", IntegrationProvider::Custom),
        id: row.get("id"),
        name: row.get("name"),
        enabled: row.get("enabled"),
        external_account_id: row.get("external_account_id"),
        webhook_url: row.get("webhook_url"),
        notes: row.get("notes"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
}

fn row_to_receipt(row: sqlx::postgres::PgRow) -> Result<Receipt, StoreError> {
    Ok(Receipt {
        kind: enum_column(&row, "kind", ReceiptKind::Deposit),
        id: row.get("id"),
        charge_id: row.get("charge_id"),
        order_id: row.get("order_id"),
        user_id: row.get("user_id"),
        amount_cents: row.get("amount_cents"),
        currency: row.get("currency"),
        occurred_at: row.get::<DateTime<Utc>, _>("occurred_at"),
        notes: row.get("notes"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
}

fn row_to_expense(row: sqlx::postgres::PgRow) -> Result<Expense, StoreError> {
    Ok(Expense {
        category: enum_column(&row, "category", ExpenseCategory::Other),
        id: row.get("id"),
        expense_date: row.get::<NaiveDate, _>("expense_date"),
        description: row.get("description"),
        vendor: row.get("vendor"),
        amount_cents: row.get("amount_cents"),
        currency: row.get("currency"),
        receipt_uri: row.get("receipt_uri"),
        bill_id: row.get("bill_id"),
        order_id: row.get("order_id"),
        notes: row.get("notes"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_store() -> AccountingStore {
        AccountingStore::connect_empty()
            .await
            .expect("PostgreSQL required for tests")
    }

    fn date(text: &str) -> NaiveDate {
        text.parse().expect("valid test date")
    }

    fn sample_line_items() -> Vec<BillLineItem> {
        vec![BillLineItem {
            sku_id: None,
            description: "Office supplies".to_string(),
            quantity: 1,
            unit_price_cents: 2500,
        }]
    }

    #[tokio::test]
    async fn create_digital_bill() {
        let store = test_store().await;
        let bill = store
            .create_bill(CreateBill {
                kind: BillKind::Digital,
                status: Some(BillStatus::Draft),
                vendor: "Acme Corp".to_string(),
                invoice_number: Some("INV-100".to_string()),
                order_id: Some("order-42".to_string()),
                bill_date: date("2026-01-15"),
                due_date: None,
                currency: None,
                line_items: sample_line_items(),
                scan_uri: None,
                notes: None,
            })
            .await
            .unwrap();
        assert_eq!(bill.vendor, "Acme Corp");
        assert_eq!(bill.kind, BillKind::Digital);
        assert_eq!(bill.total_cents, 2500);
        assert_eq!(bill.order_id.as_deref(), Some("order-42"));
        let fetched = store.get_bill(&bill.id).await.unwrap().unwrap();
        assert_eq!(fetched.order_id.as_deref(), Some("order-42"));
        assert_eq!(fetched.bill_date, date("2026-01-15"));
    }

    #[tokio::test]
    async fn scanned_bill_requires_scan_uri() {
        let store = test_store().await;
        let err = store
            .create_bill(CreateBill {
                kind: BillKind::Scanned,
                status: None,
                vendor: "Vendor".to_string(),
                invoice_number: None,
                order_id: None,
                bill_date: date("2026-01-15"),
                due_date: None,
                currency: None,
                line_items: sample_line_items(),
                scan_uri: None,
                notes: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::ScanUriRequired));
    }

    #[tokio::test]
    async fn create_expense_roundtrip() {
        let store = test_store().await;
        let bill = store
            .create_bill(CreateBill {
                kind: BillKind::Digital,
                status: None,
                vendor: "Acme Corp".to_string(),
                invoice_number: None,
                order_id: None,
                bill_date: date("2026-02-01"),
                due_date: None,
                currency: None,
                line_items: sample_line_items(),
                scan_uri: None,
                notes: None,
            })
            .await
            .unwrap();
        let expense = store
            .create_expense(CreateExpense {
                expense_date: date("2026-02-03"),
                category: ExpenseCategory::Materials,
                description: "Aluminum stock".to_string(),
                vendor: Some("Metal Supply Co".to_string()),
                amount_cents: 1250,
                currency: None,
                receipt_uri: Some("receipts/2026/alu.pdf".to_string()),
                bill_id: Some(bill.id.clone()),
                order_id: Some("order-9".to_string()),
                notes: None,
            })
            .await
            .unwrap();
        assert_eq!(expense.category, ExpenseCategory::Materials);
        assert_eq!(expense.currency, "USD");
        // Compare all but updated_at: Postgres stores microseconds, the
        // in-memory timestamp has nanoseconds.
        let fetched = store.get_expense(&expense.id).await.unwrap().unwrap();
        let expense = Expense {
            updated_at: fetched.updated_at,
            ..expense
        };
        assert_eq!(fetched, expense);
        let listed = store.list_expenses().await.unwrap();
        assert_eq!(listed.len(), 1);
    }

    #[tokio::test]
    async fn create_expense_rejects_bad_input() {
        let store = test_store().await;
        let base = CreateExpense {
            expense_date: date("2026-02-03"),
            category: ExpenseCategory::Other,
            description: "Misc".to_string(),
            vendor: None,
            amount_cents: 100,
            currency: None,
            receipt_uri: None,
            bill_id: None,
            order_id: None,
            notes: None,
        };
        let err = store
            .create_expense(CreateExpense {
                amount_cents: 0,
                ..base.clone()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::InvalidAmount));
        let err = store
            .create_expense(CreateExpense {
                description: "  ".to_string(),
                ..base.clone()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::ExpenseDescriptionRequired));
        let err = store
            .create_expense(CreateExpense {
                bill_id: Some("no-such-bill".to_string()),
                ..base
            })
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::LinkedBillNotFound));
    }

    #[tokio::test]
    async fn create_integration_rejects_duplicate_name() {
        let store = test_store().await;
        let integration = store
            .create_integration(CreateIntegration {
                name: "QuickBooks Production".to_string(),
                provider: IntegrationProvider::QuickBooks,
                enabled: Some(true),
                external_account_id: Some("qb-123".to_string()),
                webhook_url: None,
                notes: None,
            })
            .await
            .unwrap();
        assert_eq!(integration.name, "QuickBooks Production");
        assert!(integration.enabled);
        let err = store
            .create_integration(CreateIntegration {
                name: "quickbooks production".to_string(),
                provider: IntegrationProvider::Xero,
                enabled: None,
                external_account_id: None,
                webhook_url: None,
                notes: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::DuplicateIntegrationName));
    }

    #[tokio::test]
    async fn record_receipt_is_idempotent_on_charge_id() {
        let store = test_store().await;
        let input = CreateReceipt {
            charge_id: "charge-1".to_string(),
            order_id: Some("order-1".to_string()),
            user_id: "user-1".to_string(),
            kind: ReceiptKind::Deposit,
            amount_cents: 5000,
            currency: Some("usd".to_string()),
            occurred_at: None,
            notes: None,
        };
        let (receipt, created) = store.record_receipt(input.clone()).await.unwrap();
        assert!(created);
        assert_eq!(receipt.currency, "USD");
        assert_eq!(receipt.kind, ReceiptKind::Deposit);

        // Same charge again: no second row, and the original is returned.
        let (again, created_again) = store.record_receipt(input).await.unwrap();
        assert!(!created_again);
        assert_eq!(again.id, receipt.id);
        assert_eq!(store.list_receipts().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn record_receipt_rejects_bad_input() {
        let store = test_store().await;
        let base = CreateReceipt {
            charge_id: "charge-2".to_string(),
            order_id: None,
            user_id: "user-1".to_string(),
            kind: ReceiptKind::Deposit,
            amount_cents: 100,
            currency: None,
            occurred_at: None,
            notes: None,
        };
        let err = store
            .record_receipt(CreateReceipt {
                charge_id: "  ".to_string(),
                ..base.clone()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::ReceiptChargeRequired));
        let err = store
            .record_receipt(CreateReceipt {
                user_id: String::new(),
                ..base.clone()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::ReceiptUserRequired));
        let err = store
            .record_receipt(CreateReceipt {
                amount_cents: 0,
                ..base
            })
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::InvalidAmount));
    }

    #[tokio::test]
    async fn receipt_round_trip_and_delete() {
        let store = test_store().await;
        let occurred_at = "2026-03-01T12:00:00Z"
            .parse::<DateTime<Utc>>()
            .expect("valid test timestamp");
        let (receipt, _) = store
            .record_receipt(CreateReceipt {
                charge_id: "charge-3".to_string(),
                order_id: None,
                user_id: "user-2".to_string(),
                kind: ReceiptKind::Refund,
                amount_cents: 250,
                currency: None,
                occurred_at: Some(occurred_at),
                notes: Some("returned item".to_string()),
            })
            .await
            .unwrap();
        let fetched = store.get_receipt(&receipt.id).await.unwrap().unwrap();
        assert_eq!(fetched.kind, ReceiptKind::Refund);
        assert_eq!(fetched.occurred_at, occurred_at);
        assert_eq!(fetched.notes.as_deref(), Some("returned item"));

        let by_charge = store
            .get_receipt_by_charge("charge-3")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(by_charge.id, receipt.id);

        store.delete_receipt(&receipt.id).await.unwrap();
        let err = store.delete_receipt(&receipt.id).await.unwrap_err();
        assert!(matches!(err, StoreError::ReceiptNotFound));
    }
}
