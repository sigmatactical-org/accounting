//! Catalog SKU lookups — a thin wrapper over the shared catalog client.

use std::sync::Arc;

pub use sigma_pg::clients::catalog::{CatalogError, CatalogSku};

/// All catalog SKUs. The shared client caches per process with a short TTL,
/// so callers may call this once per page render.
///
/// # Errors
///
/// [`CatalogError::NotConfigured`] when `ACCOUNTING_CATALOG_BASE_URL` is
/// unset; other variants when the catalog service can't be reached.
pub async fn fetch_skus() -> Result<Arc<Vec<CatalogSku>>, CatalogError> {
    sigma_pg::clients::catalog::fetch_skus(crate::config::catalog_base_url().as_deref()).await
}
