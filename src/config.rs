fn normalize_base_url(url: String) -> String {
    let mut url = url.trim().to_string();
    if !url.ends_with('/') {
        url.push('/');
    }
    url
}

/// Public base URL of this accounting service (e.g. `http://127.0.0.1:8080/`).
#[must_use]
pub fn public_base_url() -> String {
    std::env::var("ACCOUNTING_PUBLIC_BASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(normalize_base_url)
        .unwrap_or_else(|| "http://127.0.0.1:8080/".to_string())
}

/// Public base URL of the identity BFF (e.g. `http://127.0.0.1:3000/`).
#[must_use]
pub fn identity_public_base_url() -> String {
    std::env::var("ACCOUNTING_IDENTITY_PUBLIC_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(normalize_base_url)
        .unwrap_or_else(|| "http://127.0.0.1:3000/".to_string())
}

/// Browser origin of the identity BFF for CSP `connect-src` (no trailing slash).
#[must_use]
pub fn identity_public_origin() -> String {
    identity_public_base_url().trim_end_matches('/').to_string()
}

/// Public base URL of the contact service for navbar links.
#[must_use]
pub fn contact_public_base_url() -> String {
    std::env::var("ACCOUNTING_CONTACT_PUBLIC_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(normalize_base_url)
        .unwrap_or_else(|| "http://127.0.0.1:8083/".to_string())
}

/// Public base URL of the cart service for navbar links.
#[must_use]
pub fn cart_public_base_url() -> String {
    std::env::var("ACCOUNTING_CART_PUBLIC_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(normalize_base_url)
        .unwrap_or_else(|| "http://127.0.0.1:8084/".to_string())
}

/// Base URL of the catalog service (e.g. `http://127.0.0.1:8081/`).
#[must_use]
pub fn catalog_base_url() -> Option<String> {
    std::env::var("ACCOUNTING_CATALOG_BASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            let mut url = s.trim().to_string();
            if !url.ends_with('/') {
                url.push('/');
            }
            url
        })
}

/// Whether catalog integration is configured.
#[must_use]
pub fn catalog_configured() -> bool {
    catalog_base_url().is_some()
}

/// PostgreSQL connection URL (shared Sigma database).
#[must_use]
pub fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| sigma_pg::service_database_url("accounting"))
}
