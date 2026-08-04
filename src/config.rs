//! Environment-driven configuration (service URLs, database URL).
//!
//! Required values are declared in the [`sigma_config::service!`] block and
//! checked by [`validate_with`] at startup; optional integrations return
//! `None` when they are not configured for this environment.

sigma_config::service! {
    prefix = "ACCOUNTING";
    role = "accounting";
    urls {
        /// Public base URL of this accounting service.
        public_base_url = "PUBLIC_BASE_URL" => "http://127.0.0.1:8080/";
        /// Public base URL of the identity BFF.
        identity_public_base_url = "IDENTITY_PUBLIC_URL" => "http://127.0.0.1:3000/";
        /// Public base URL of the contact service, for navbar links.
        contact_public_base_url = "CONTACT_PUBLIC_URL" => "http://127.0.0.1:8083/";
        /// Public base URL of the cart service, for navbar links.
        cart_public_base_url = "CART_PUBLIC_URL" => "http://127.0.0.1:8084/";
    }
}

/// Browser origin of the identity BFF for CSP `connect-src` (no trailing slash).
#[must_use]
pub fn identity_public_origin() -> String {
    sigma_config::origin_of(&identity_public_base_url())
}

/// Base URL of the catalog service (e.g. `http://127.0.0.1:8081/`).
#[must_use]
pub fn catalog_base_url() -> Option<String> {
    SERVICE.opt_url("CATALOG_BASE_URL")
}

/// Whether catalog integration is configured.
#[must_use]
pub fn catalog_configured() -> bool {
    catalog_base_url().is_some()
}

/// Base URL of the orders service's internal API (e.g. `http://127.0.0.1:8085/`).
#[must_use]
pub fn orders_base_url() -> Option<String> {
    SERVICE.opt_url("ORDERS_BASE_URL")
}

/// Public base URL of the orders admin UI, for bill order links.
#[must_use]
pub fn orders_public_base_url() -> Option<String> {
    SERVICE.opt_url("ORDERS_PUBLIC_URL")
}

/// Base URL of the payments service's internal API (e.g. `http://127.0.0.1:8090/`).
#[must_use]
pub fn payments_base_url() -> Option<String> {
    SERVICE.opt_url("PAYMENTS_BASE_URL")
}

/// Whether payments integration (receipt reconcile) is configured.
#[must_use]
pub fn payments_configured() -> bool {
    payments_base_url().is_some()
}

/// PostgreSQL connection URL (shared Sigma database).
#[must_use]
pub fn database_url() -> String {
    SERVICE.database_url()
}
