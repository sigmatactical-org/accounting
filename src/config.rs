/// PostgreSQL connection URL (shared Sigma database).
#[must_use]
pub fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| sigma_pg::service_database_url("accounting"))
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
