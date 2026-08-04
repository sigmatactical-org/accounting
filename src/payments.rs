//! Client for the payments service's internal-token-gated JSON API
//! (`GET /api/charges`), plus the receipt reconcile backstop.
//!
//! The cart records a receipt at checkout, but that push is best-effort — it
//! must never fail a paid checkout. Reconcile is the safety net: it sweeps
//! the payments charge log and records a receipt for every successful charge
//! that doesn't have one yet. [`crate::store::AccountingStore::record_receipt`]
//! is idempotent on charge id, so the two paths can overlap freely and
//! reconcile can be re-run at any time.
//!
//! Accounting and payments are independently owned services communicating
//! over HTTP+JSON only — receipts store the charge id as an opaque
//! reference, never a database foreign key.

mod charge_summary;
mod payments_error;
mod reconcile_outcome;
pub use charge_summary::ChargeSummary;
pub use payments_error::PaymentsError;
pub use reconcile_outcome::ReconcileOutcome;

use std::collections::HashMap;

use sigma_pg::clients::http;

use crate::model::{CreateReceipt, ReceiptKind};
use crate::store::{AccountingStore, StoreError};

fn build_charges_url(base: &str) -> String {
    format!("{base}api/charges")
}

/// Every charge recorded by the payments service.
async fn fetch_charges(base: &str) -> Result<Vec<ChargeSummary>, PaymentsError> {
    let url = build_charges_url(base);
    let response = http::with_internal_auth(http::client().get(url))
        .send()
        .await?;
    let response = http::ensure_success(response)
        .await
        .map_err(PaymentsError::Request)?;
    Ok(response.json().await?)
}

/// Map of charge id → order id, from the orders service.
///
/// The charge log this reconciles against does not expose the payment
/// reference, so the charge → order link is read from the order rows, which
/// record the charge that paid their deposit. Best-effort: an unconfigured or
/// unreachable orders service yields an empty map and reconciled receipts
/// simply carry no order link, which a later reconcile cannot repair but which
/// never blocks recording the money itself.
async fn order_ids_by_charge() -> HashMap<String, String> {
    match crate::orders::fetch_order_refs().await {
        Ok(orders) => orders
            .into_iter()
            .filter_map(|order| order.charge_id.map(|charge_id| (charge_id, order.id)))
            .collect(),
        Err(_) => HashMap::new(),
    }
}

/// Record a receipt for every successful charge that doesn't have one yet.
///
/// # Errors
///
/// [`StoreError::Payments`] when payments integration is not configured or
/// the charge log can't be fetched; database errors pass through.
pub async fn reconcile_receipts(store: &AccountingStore) -> Result<ReconcileOutcome, StoreError> {
    let base = crate::config::payments_base_url()
        .ok_or_else(|| StoreError::Payments(PaymentsError::NotConfigured.to_string()))?;
    let charges = fetch_charges(&base)
        .await
        .map_err(|e| StoreError::Payments(e.to_string()))?;
    let order_ids = order_ids_by_charge().await;

    let mut outcome = ReconcileOutcome::default();
    for charge in charges.into_iter().filter(ChargeSummary::succeeded) {
        outcome.charges_seen += 1;
        let (_, created) = store
            .record_receipt(CreateReceipt {
                order_id: order_ids.get(&charge.id).cloned(),
                charge_id: charge.id,
                user_id: charge.user_id,
                kind: ReceiptKind::Deposit,
                amount_cents: charge.amount_cents,
                currency: Some(charge.currency),
                occurred_at: Some(charge.created_at),
                notes: None,
            })
            .await?;
        if created {
            outcome.created += 1;
        } else {
            outcome.already_recorded += 1;
        }
    }
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn build_charges_url_joins_base() {
        assert_eq!(
            build_charges_url("http://payments.internal:8090/"),
            "http://payments.internal:8090/api/charges"
        );
    }

    #[test]
    fn only_succeeded_charges_become_receipts() {
        let charge = |status: &str| ChargeSummary {
            id: "charge-1".to_string(),
            user_id: "user-1".to_string(),
            amount_cents: 5000,
            currency: "usd".to_string(),
            status: status.to_string(),
            created_at: Utc::now(),
        };
        assert!(charge("succeeded").succeeded());
        assert!(!charge("failed").succeeded());
    }
}
