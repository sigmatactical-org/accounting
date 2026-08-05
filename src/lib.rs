//! Sigma Accounting: bills, expenses, integrations, and catalog-linked
//! line items.

#![forbid(unsafe_code)]

mod api;
pub mod catalog;
pub mod config;
mod model;
pub mod orders;
pub mod payments;
pub mod store;
mod session_status;
mod templates;
mod web;

use std::convert::Infallible;
use std::sync::Arc;

use warp::Filter;
use warp::Reply;

pub use model::{
    Bill, BillKind, BillLineItem, BillStatus, CreateBill, CreateExpense, CreateIntegration,
    CreateReceipt, Expense, ExpenseCategory, Integration, IntegrationProvider, Receipt,
    ReceiptKind, UpdateBill, UpdateExpense, UpdateIntegration,
};

/// Shared accounting store handle (`PgPool` is internally concurrent).
pub type SharedStore = Arc<store::AccountingStore>;

/// Site routes: web UI, JSON API, `/up`, health, theme static assets, and
/// themed error recovery, with the shared security headers applied.
pub fn routes(
    store: store::AccountingStore,
) -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone + Send + 'static {
    let health_pool = Arc::new(store.pool().clone());
    let store = Arc::new(store);

    let index = web::routes(sigma_theme::warp::with_state(store.clone()));
    let extra = sigma_pg::health::warp::health_routes("accounting", Some(health_pool))
        .or(api::routes(sigma_theme::warp::with_state(store)));

    sigma_theme::warp::security_headers(
        sigma_theme::warp::site_routes(index, extra),
        config::identity_public_origin(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use warp::http::StatusCode;

    async fn test_store() -> store::AccountingStore {
        sigma_pg::test_helpers::ready_store(store::AccountingStore::connect_empty()).await
    }

    fn internal_token() -> &'static str {
        sigma_pg::clients::internal::TEST_INTERNAL_TOKEN
    }

    #[tokio::test]
    async fn up_returns_ok() {
        let res = warp::test::request()
            .method("GET")
            .path("/up")
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn index_without_session_redirects_to_sign_in() {
        let res = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::SEE_OTHER);
        let location = res
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        assert!(location.contains("/auth/login"));
    }

    #[tokio::test]
    async fn api_lists_empty_bills() {
        let res = warp::test::request()
            .method("GET")
            .path("/bills")
            .header("accept", "application/json")
            .header("x-sigma-internal-token", internal_token())
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let body: Vec<Bill> = serde_json::from_slice(res.body()).unwrap();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn api_create_digital_bill() {
        let res = warp::test::request()
            .method("POST")
            .path("/bills")
            .header("content-type", "application/json")
            .header("x-sigma-internal-token", internal_token())
            .body(
                r#"{"kind":"digital","vendor":"Acme Corp","invoice_number":"INV-1","order_id":"order-7","bill_date":"2026-01-15","line_items":[{"description":"Supplies","quantity":1,"unit_price_cents":2500}]}"#,
            )
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let bill: Bill = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(bill.vendor, "Acme Corp");
        assert_eq!(bill.kind, BillKind::Digital);
        assert_eq!(bill.total_cents, 2500);
        assert_eq!(bill.order_id.as_deref(), Some("order-7"));
        assert_eq!(bill.bill_date.to_string(), "2026-01-15");
    }

    #[tokio::test]
    async fn api_create_expense() {
        let res = warp::test::request()
            .method("POST")
            .path("/expenses")
            .header("content-type", "application/json")
            .header("x-sigma-internal-token", internal_token())
            .body(
                r#"{"expense_date":"2026-02-03","category":"materials","description":"Aluminum stock","vendor":"Metal Supply Co","amount_cents":1250,"order_id":"order-9"}"#,
            )
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let expense: Expense = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(expense.category, ExpenseCategory::Materials);
        assert_eq!(expense.amount_cents, 1250);
        assert_eq!(expense.order_id.as_deref(), Some("order-9"));
    }

    /// Recording the same charge twice must not double-count revenue: the
    /// second POST answers `200` with the original receipt, not `201`.
    #[tokio::test]
    async fn api_record_receipt_is_idempotent_on_charge_id() {
        let routes = routes(test_store().await);
        let body = r#"{"charge_id":"charge-1","order_id":"order-1","user_id":"user-1","kind":"deposit","amount_cents":5000,"currency":"usd"}"#;
        let post = || {
            warp::test::request()
                .method("POST")
                .path("/receipts")
                .header("content-type", "application/json")
                .header("x-sigma-internal-token", internal_token())
                .body(body)
        };

        let res = post().reply(&routes).await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let created: Receipt = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(created.kind, ReceiptKind::Deposit);
        assert_eq!(created.amount_cents, 5000);
        assert_eq!(created.currency, "USD");

        let res = post().reply(&routes).await;
        assert_eq!(res.status(), StatusCode::OK);
        let existing: Receipt = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(existing.id, created.id);

        let res = warp::test::request()
            .method("GET")
            .path("/receipts")
            .header("x-sigma-internal-token", internal_token())
            .reply(&routes)
            .await;
        let listed: Vec<Receipt> = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(listed.len(), 1);
    }

    #[tokio::test]
    async fn api_create_expense_rejects_unknown_bill_link() {
        let res = warp::test::request()
            .method("POST")
            .path("/expenses")
            .header("content-type", "application/json")
            .header("x-sigma-internal-token", internal_token())
            .body(
                r#"{"expense_date":"2026-02-03","category":"fees","description":"Card fee","amount_cents":95,"bill_id":"no-such-bill"}"#,
            )
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = std::str::from_utf8(res.body()).unwrap();
        assert!(body.contains("linked bill not found"));
    }

    #[tokio::test]
    async fn api_create_integration() {
        let res = warp::test::request()
            .method("POST")
            .path("/integrations")
            .header("content-type", "application/json")
            .header("x-sigma-internal-token", internal_token())
            .body(r#"{"name":"QuickBooks","provider":"quickbooks","enabled":true}"#)
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let integration: Integration = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(integration.name, "QuickBooks");
        assert_eq!(integration.provider, IntegrationProvider::QuickBooks);
    }

    /// A duplicate integration name is a conflict, not a bad request.
    #[tokio::test]
    async fn api_duplicate_integration_name_conflicts() {
        let routes = routes(test_store().await);
        let body = r#"{"name":"QuickBooks","provider":"quickbooks","enabled":true}"#;
        let create = || {
            warp::test::request()
                .method("POST")
                .path("/integrations")
                .header("content-type", "application/json")
                .header("x-sigma-internal-token", internal_token())
                .body(body)
                .reply(&routes)
        };
        assert_eq!(create().await.status(), StatusCode::CREATED);
        let res = create().await;
        assert_eq!(res.status(), StatusCode::CONFLICT);
        let body = std::str::from_utf8(res.body()).unwrap();
        assert!(body.contains("integration name already exists"));
    }

    #[tokio::test]
    async fn catalog_skus_not_configured() {
        let res = warp::test::request()
            .method("GET")
            .path("/catalog/skus")
            .header("x-sigma-internal-token", internal_token())
            .reply(&routes(test_store().await))
            .await;
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
