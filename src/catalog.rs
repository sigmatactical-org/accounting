pub use sigma_pg::clients::catalog::{
    CatalogError, CatalogSku, CatalogSkuComponent, CatalogSkuKind,
};

pub async fn fetch_skus() -> Result<Vec<CatalogSku>, CatalogError> {
    sigma_pg::clients::catalog::fetch_skus(crate::config::catalog_base_url().as_deref()).await
}

pub use sigma_pg::clients::catalog::{sku_by_id, validate_sku_id};

#[must_use]
pub fn sku_code_by_id(skus: &[CatalogSku], id: &str) -> Option<String> {
    sku_by_id(skus, id).map(|s| s.sku_code.clone())
}
