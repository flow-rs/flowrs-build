use anyhow::{Context, Result};
use std::net::SocketAddr;

use flowrs_build::{
    service::config::load_environment_and_configure, service::server::setup_server,
};

use tokio::sync::broadcast;

async fn handle_shutdown_signal(stopper: broadcast::Sender<()>) -> std::io::Result<()> {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to set up Ctrl+C handler");
    println!("-> Shutdown requested.");
    let _ = stopper.send(());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    //Spawn async Task for stopping the service
    let (stopper_sender, _) = broadcast::channel::<()>(1);
    let stopper_sender_clone = stopper_sender.clone();
    tokio::spawn(handle_shutdown_signal(stopper_sender_clone));

    //loads arguments, environment and config
    let config = load_environment_and_configure();
    let addr = SocketAddr::new(config.ip, config.port);

    //setup server
    let app = setup_server(config.clone()).context("Failed to setup server")?;
    println!("-> Listening on {}", addr);
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            stopper_sender.subscribe().recv().await.ok();
        });

    // Run the server until it's gracefully shut down
    let _ = server.await;

    println!("-> Service shut down.");

    Ok(())
}
