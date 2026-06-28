type BoxError = Box<dyn std::error::Error + Send + Sync>;

fn main() -> Result<(), BoxError> {
    let addr = sigma_accounting::listen_socket_addr_from_env();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let store = sigma_accounting::store::AccountingStore::load(
                sigma_accounting::config::data_path(),
            )?;
            let listener = tokio::net::TcpListener::bind(addr).await?;
            println!("Sigma Accounting listening on http://{addr}");
            warp::serve(sigma_accounting::routes(store))
                .incoming(listener)
                .graceful(shutdown_signal())
                .run()
                .await;
            Ok::<(), BoxError>(())
        })
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let (mut term, mut int) = match (
            signal(SignalKind::terminate()),
            signal(SignalKind::interrupt()),
        ) {
            (Ok(term), Ok(int)) => (term, int),
            _ => {
                eprintln!("warning: could not install signal handlers; graceful shutdown disabled");
                std::future::pending::<()>().await;
                return;
            }
        };
        tokio::select! {
            _ = term.recv() => {}
            _ = int.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
