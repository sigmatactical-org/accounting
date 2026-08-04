#![forbid(unsafe_code)]

type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn main() -> Result<(), BoxError> {
    sigma_accounting::config::validate_with(&[sigma_config::DATABASE_URL_VAR])?;
    let addr = sigma_theme::warp::listen_addr_from_env();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let store = sigma_accounting::store::AccountingStore::connect().await?;
            sigma_theme::warp::serve("Sigma Accounting", addr, sigma_accounting::routes(store))
                .await?;
            Ok::<(), BoxError>(())
        })
}
