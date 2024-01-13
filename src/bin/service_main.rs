use anyhow::{Context, Result};
use std::net::SocketAddr;

use flowrs_build::{
    service::config::load_environment_and_configure, service::server::setup_server,
};

use tokio::{net::TcpListener, signal};

//https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs#L51
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    //loads arguments, environment and config
    let config = load_environment_and_configure();
    let addr = SocketAddr::new(config.ip, config.port);

    //setup server
    let listener = TcpListener::bind(&addr).await.unwrap();
    let app = setup_server(config.clone()).context("Failed to setup server")?;
    println!("-> Listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    println!("-> Service shut down.");

    Ok(())
}
