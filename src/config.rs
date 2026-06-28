use std::path::PathBuf;

/// Path to the JSON accounting database.
#[must_use]
pub fn data_path() -> PathBuf {
    std::env::var("ACCOUNTING_DATA_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data/accounting.json"))
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
