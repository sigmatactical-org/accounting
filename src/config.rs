//! Environment-driven configuration (service URLs, database URL).

use sigma_pg::clients::http::{env_url, normalize_base_url};

/// Base URL from `var`, or `None` when the variable is unset or blank.
fn optional_env_url(var: &str) -> Option<String> {
    std::env::var(var)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| normalize_base_url(&s))
}

/// Public base URL of this accounting service (e.g. `http://127.0.0.1:8080/`).
#[must_use]
pub fn public_base_url() -> String {
    env_url("ACCOUNTING_PUBLIC_BASE_URL", "http://127.0.0.1:8080/")
}

/// Public base URL of the identity BFF (e.g. `http://127.0.0.1:3000/`).
#[must_use]
pub fn identity_public_base_url() -> String {
    env_url("ACCOUNTING_IDENTITY_PUBLIC_URL", "http://127.0.0.1:3000/")
}

/// Browser origin of the identity BFF for CSP `connect-src` (no trailing slash).
#[must_use]
pub fn identity_public_origin() -> String {
    identity_public_base_url().trim_end_matches('/').to_string()
}

/// Public base URL of the contact service for navbar links.
#[must_use]
pub fn contact_public_base_url() -> String {
    env_url("ACCOUNTING_CONTACT_PUBLIC_URL", "http://127.0.0.1:8083/")
}

/// Public base URL of the cart service for navbar links.
#[must_use]
pub fn cart_public_base_url() -> String {
    env_url("ACCOUNTING_CART_PUBLIC_URL", "http://127.0.0.1:8084/")
}

/// Base URL of the catalog service (e.g. `http://127.0.0.1:8081/`).
#[must_use]
pub fn catalog_base_url() -> Option<String> {
    optional_env_url("ACCOUNTING_CATALOG_BASE_URL")
}

/// Whether catalog integration is configured.
#[must_use]
pub fn catalog_configured() -> bool {
    catalog_base_url().is_some()
}

/// Base URL of the orders service's internal API (e.g. `http://127.0.0.1:8085/`).
#[must_use]
pub fn orders_base_url() -> Option<String> {
    optional_env_url("ACCOUNTING_ORDERS_BASE_URL")
}

/// Public base URL of the orders admin UI, for bill order links.
#[must_use]
pub fn orders_public_base_url() -> Option<String> {
    optional_env_url("ACCOUNTING_ORDERS_PUBLIC_URL")
}

/// Base URL of the payments service's internal API (e.g. `http://127.0.0.1:8090/`).
#[must_use]
pub fn payments_base_url() -> Option<String> {
    optional_env_url("ACCOUNTING_PAYMENTS_BASE_URL")
}

/// Whether payments integration (receipt reconcile) is configured.
#[must_use]
pub fn payments_configured() -> bool {
    payments_base_url().is_some()
}

/// PostgreSQL connection URL (shared Sigma database).
#[must_use]
pub fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| sigma_pg::service_database_url("accounting"))
}
