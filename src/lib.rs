//! Sigma Accounting: bills, integrations, and catalog-linked line items.

mod api;
pub mod catalog;
pub mod config;
mod model;
pub mod store;
mod templates;
mod web;

use std::convert::Infallible;
use std::sync::Arc;

use tokio::sync::Mutex;
use warp::Filter;
use warp::Reply;

pub use model::{
    Bill, BillKind, BillLineItem, BillStatus, CreateBill, CreateIntegration, Integration,
    IntegrationProvider, UpdateBill, UpdateIntegration,
};

/// Shared mutable accounting store handle.
pub type SharedStore = Arc<Mutex<store::AccountingStore>>;

/// Resolve listen address from **`PORT`** (default **8080**).
#[must_use]
pub fn listen_socket_addr_from_env() -> std::net::SocketAddr {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)
}

fn with_store(
    store: SharedStore,
) -> impl Filter<Extract = (SharedStore,), Error = Infallible> + Clone {
    warp::any().map(move || store.clone())
}

/// Site routes: web UI, JSON API, `/up`, theme static assets, error recovery.
pub fn routes(
    store: store::AccountingStore,
) -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone + Send + 'static {
    use warp::reply::with::header;

    let store = Arc::new(Mutex::new(store));

    warp::path("up")
        .and(warp::get())
        .map(|| warp::reply::with_status("up", warp::http::StatusCode::OK))
        .or(web::routes(with_store(store.clone())))
        .or(api::routes(with_store(store)))
        .or(sigma_theme::warp::static_files())
        .or(sigma_theme::warp::favicon())
        .recover(sigma_theme::warp::handle_rejection)
        .with(header(
            "content-security-policy",
            "default-src 'self'; base-uri 'self'; object-src 'none'; frame-ancestors 'none'; \
             img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; \
             font-src 'self'; connect-src 'self'; form-action 'self'",
        ))
        .with(header("x-content-type-options", "nosniff"))
        .with(header("x-frame-options", "DENY"))
        .with(header("referrer-policy", "strict-origin-when-cross-origin"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use warp::http::StatusCode;

    fn test_store() -> store::AccountingStore {
        let dir = TempDir::new().unwrap();
        store::AccountingStore::load(dir.path().join("accounting.json")).unwrap()
    }

    #[tokio::test]
    async fn up_returns_ok() {
        let res = warp::test::request()
            .method("GET")
            .path("/up")
            .reply(&routes(test_store()))
            .await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn index_lists_accounting() {
        let res = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&routes(test_store()))
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let body = std::str::from_utf8(res.body()).unwrap();
        assert!(body.contains("Accounting"));
    }

    #[tokio::test]
    async fn api_lists_empty_bills() {
        let res = warp::test::request()
            .method("GET")
            .path("/bills")
            .header("accept", "application/json")
            .reply(&routes(test_store()))
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
            .body(
                r#"{"kind":"digital","vendor":"Acme Corp","invoice_number":"INV-1","bill_date":"2026-01-15","line_items":[{"description":"Supplies","quantity":1,"unit_price_cents":2500}]}"#,
            )
            .reply(&routes(test_store()))
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let bill: Bill = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(bill.vendor, "Acme Corp");
        assert_eq!(bill.kind, BillKind::Digital);
        assert_eq!(bill.total_cents, 2500);
    }

    #[tokio::test]
    async fn api_create_integration() {
        let res = warp::test::request()
            .method("POST")
            .path("/integrations")
            .header("content-type", "application/json")
            .body(r#"{"name":"QuickBooks","provider":"quickbooks","enabled":true}"#)
            .reply(&routes(test_store()))
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let integration: Integration = serde_json::from_slice(res.body()).unwrap();
        assert_eq!(integration.name, "QuickBooks");
        assert_eq!(integration.provider, IntegrationProvider::QuickBooks);
    }

    #[tokio::test]
    async fn catalog_skus_not_configured() {
        let res = warp::test::request()
            .method("GET")
            .path("/catalog/skus")
            .reply(&routes(test_store()))
            .await;
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
