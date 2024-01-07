use anyhow::{Context, Result};
use std::net::SocketAddr;

use flowrs_build::{
    service::config::load_environment_and_configure, service::server::setup_server,
};

use tokio::{net::TcpListener, signal};

// async fn handle_shutdown_signal(stopper: broadcast::Sender<()>) -> std::io::Result<()> {
//     tokio::signal::ctrl_c()
//         .await
//         .expect("Failed to set up Ctrl+C handler");
//     println!("-> Shutdown requested.");
//     let _ = stopper.send(());
//     Ok(())
// }

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
    //Spawn async Task for stopping the service
    //let (stopper_sender, _) = broadcast::channel::<()>(1);
    //let stopper_sender_clone = stopper_sender.clone();
    //tokio::spawn(handle_shutdown_signal(stopper_sender_clone));

    //loads arguments, environment and config
    let config = load_environment_and_configure();
    let addr = SocketAddr::new(config.ip, config.port);

    //setup server
    let listener = TcpListener::bind(&addr).await.unwrap();
    let app = setup_server(config.clone()).context("Failed to setup server")?;
    println!("-> Listening on {}", addr);
    // let server = Server::bind(&addr)
    //     .serve(app.into_make_service())
    //     .with_graceful_shutdown(async {
    //         stopper_sender.subscribe().recv().await.ok();
    //     });

    // // Run the server until it's gracefully shut down
    // let _ = server.await;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    println!("-> Service shut down.");

    Ok(())
}
