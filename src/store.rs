use sqlx::PgPool;
use thiserror::Error;

use crate::model::{
    Bill, BillKind, CreateBill, CreateIntegration, Integration, UpdateBill, UpdateIntegration,
    compute_total_cents,
};

const SCHEMA: &str = "accounting";

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("bill not found")]
    BillNotFound,
    #[error("integration not found")]
    IntegrationNotFound,
    #[error("vendor is required")]
    VendorRequired,
    #[error("bill date is required")]
    BillDateRequired,
    #[error("bill must have at least one line item")]
    BillNeedsLineItems,
    #[error("scanned bill requires scan_uri")]
    ScanUriRequired,
    #[error("line item quantity must be at least 1")]
    InvalidQuantity,
    #[error("integration name is required")]
    IntegrationNameRequired,
    #[error("integration name already exists")]
    DuplicateIntegrationName,
    #[error("database error: {0}")]
    Database(#[from] anyhow::Error),
    #[error("{0}")]
    InvalidInput(String),
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Database {
    bills: Vec<Bill>,
    integrations: Vec<Integration>,
}

#[derive(Debug, Clone)]
pub struct AccountingStore {
    pool: PgPool,
    db: Database,
}

impl AccountingStore {
    /// Connect to PostgreSQL and load the accounting snapshot.
    pub async fn connect() -> Result<Self, StoreError> {
        let pool = sigma_pg::connect().await?;
        let db: Database = sigma_pg::load_snapshot(&pool, SCHEMA).await?;
        Ok(Self { pool, db })
    }

    /// Reset the accounting snapshot (tests only).
    #[cfg(test)]
    pub async fn connect_empty() -> Result<Self, StoreError> {
        let pool = sigma_pg::connect().await?;
        let db = Database::default();
        sigma_pg::save_snapshot(&pool, SCHEMA, &db).await?;
        Ok(Self { pool, db })
    }

    async fn persist(&self) -> Result<(), StoreError> {
        sigma_pg::save_snapshot(&self.pool, SCHEMA, &self.db).await?;
        Ok(())
    }

    #[must_use]
    pub fn list_bills(&self) -> Vec<Bill> {
        let mut bills = self.db.bills.clone();
        bills.sort_by(|a, b| b.bill_date.cmp(&a.bill_date));
        bills
    }

    #[must_use]
    pub fn get_bill(&self, id: &str) -> Option<Bill> {
        self.db.bills.iter().find(|b| b.id == id).cloned()
    }

    pub async fn create_bill(&mut self, input: CreateBill) -> Result<Bill, StoreError> {
        self.validate_bill_input(
            input.kind,
            &input.vendor,
            &input.bill_date,
            &input.line_items,
            input.scan_uri.as_deref(),
        )?;
        let bill = Bill::new(input);
        self.db.bills.push(bill.clone());
        self.persist().await?;
        Ok(bill)
    }

    pub async fn update_bill(&mut self, id: &str, input: UpdateBill) -> Result<Bill, StoreError> {
        if self.get_bill(id).is_none() {
            return Err(StoreError::BillNotFound);
        }
        self.validate_bill_input(
            input.kind,
            &input.vendor,
            &input.bill_date,
            &input.line_items,
            input.scan_uri.as_deref(),
        )?;
        let bill = self
            .db
            .bills
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or(StoreError::BillNotFound)?;
        bill.apply_update(input);
        let updated = bill.clone();
        self.persist().await?;
        Ok(updated)
    }

    pub async fn delete_bill(&mut self, id: &str) -> Result<(), StoreError> {
        let index = self
            .db
            .bills
            .iter()
            .position(|b| b.id == id)
            .ok_or(StoreError::BillNotFound)?;
        self.db.bills.remove(index);
        self.persist().await
    }

    #[must_use]
    pub fn list_integrations(&self) -> Vec<Integration> {
        let mut integrations = self.db.integrations.clone();
        integrations.sort_by_key(|i| i.name.to_lowercase());
        integrations
    }

    #[must_use]
    pub fn get_integration(&self, id: &str) -> Option<Integration> {
        self.db.integrations.iter().find(|i| i.id == id).cloned()
    }

    pub async fn create_integration(
        &mut self,
        input: CreateIntegration,
    ) -> Result<Integration, StoreError> {
        self.validate_integration_name(&input.name, None)?;
        let integration = Integration::new(input);
        self.db.integrations.push(integration.clone());
        self.persist().await?;
        Ok(integration)
    }

    pub async fn update_integration(
        &mut self,
        id: &str,
        input: UpdateIntegration,
    ) -> Result<Integration, StoreError> {
        if self.get_integration(id).is_none() {
            return Err(StoreError::IntegrationNotFound);
        }
        self.validate_integration_name(&input.name, Some(id))?;
        let integration = self
            .db
            .integrations
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or(StoreError::IntegrationNotFound)?;
        integration.apply_update(input);
        let updated = integration.clone();
        self.persist().await?;
        Ok(updated)
    }

    pub async fn delete_integration(&mut self, id: &str) -> Result<(), StoreError> {
        let index = self
            .db
            .integrations
            .iter()
            .position(|i| i.id == id)
            .ok_or(StoreError::IntegrationNotFound)?;
        self.db.integrations.remove(index);
        self.persist().await
    }

    fn validate_bill_input(
        &self,
        kind: BillKind,
        vendor: &str,
        bill_date: &str,
        line_items: &[crate::model::BillLineItem],
        scan_uri: Option<&str>,
    ) -> Result<(), StoreError> {
        if vendor.trim().is_empty() {
            return Err(StoreError::VendorRequired);
        }
        if bill_date.trim().is_empty() {
            return Err(StoreError::BillDateRequired);
        }
        if line_items.is_empty() {
            return Err(StoreError::BillNeedsLineItems);
        }
        for item in line_items {
            if item.quantity == 0 {
                return Err(StoreError::InvalidQuantity);
            }
        }
        if kind == BillKind::Scanned && scan_uri.map(str::trim).filter(|s| !s.is_empty()).is_none()
        {
            return Err(StoreError::ScanUriRequired);
        }
        let _ = compute_total_cents(line_items);
        Ok(())
    }

    fn validate_integration_name(
        &self,
        name: &str,
        except_id: Option<&str>,
    ) -> Result<(), StoreError> {
        if name.trim().is_empty() {
            return Err(StoreError::IntegrationNameRequired);
        }
        let normalized = name.trim().to_lowercase();
        if self
            .db
            .integrations
            .iter()
            .any(|i| except_id != Some(i.id.as_str()) && i.name.to_lowercase() == normalized)
        {
            return Err(StoreError::DuplicateIntegrationName);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BillKind, BillLineItem, BillStatus, IntegrationProvider};

    async fn test_store() -> AccountingStore {
        AccountingStore::connect_empty()
            .await
            .expect("PostgreSQL required for tests")
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
        let mut store = test_store().await;
        let bill = store
            .create_bill(CreateBill {
                kind: BillKind::Digital,
                status: Some(BillStatus::Draft),
                vendor: "Acme Corp".to_string(),
                invoice_number: Some("INV-100".to_string()),
                bill_date: "2026-01-15".to_string(),
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
    }

    #[tokio::test]
    async fn scanned_bill_requires_scan_uri() {
        let mut store = test_store().await;
        let err = store
            .create_bill(CreateBill {
                kind: BillKind::Scanned,
                status: None,
                vendor: "Vendor".to_string(),
                invoice_number: None,
                bill_date: "2026-01-15".to_string(),
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
    async fn create_integration() {
        let mut store = test_store().await;
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
    }
}
