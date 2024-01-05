use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{Response, StatusCode},
    routing::{get, post},
    Json, Router,
};
use tokio::sync::broadcast;

use dotenv::dotenv;

use tower_http::cors::{Any, CorsLayer};

use axum::routing::delete;
use clap::Parser;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use flowrs_build::flow_project::BuildType;
use flowrs_build::{
    flow_project::{FlowProject, FlowProjectManager, FlowProjectManagerConfig},
    package::Package,
    package_manager::PackageManager,
};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Json config file.
    #[arg(short, long, default_value_t = f("config.json"))]
    config_file: String,
}

fn f(str: &str) -> String {
    str.to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ServiceConfig {
    #[serde(default = "flow_project_manager_config_default")]
    flow_project_manager_config: FlowProjectManagerConfig,

    #[serde(default = "flow_packages_folder_default")]
    flow_packages_folder: String,
}

fn flow_project_manager_config_default() -> FlowProjectManagerConfig {
    FlowProjectManagerConfig::default()
}

fn flow_packages_folder_default() -> String {
    "flow-packages".to_string()
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            flow_project_manager_config: flow_project_manager_config_default(),
            flow_packages_folder: flow_packages_folder_default(),
        }
    }
}

fn load_config(config_path: &str) -> ServiceConfig {
    if std::path::Path::new(config_path).exists() {
        // Read and deserialize the existing config.json file
        let mut file =
            File::open(config_path).expect(&format!("Failed to open {}", config_path).as_str());
        let mut config_content = String::new();
        file.read_to_string(&mut config_content)
            .expect(&format!("Failed to read {}", config_path).as_str());
        serde_json::from_str(&config_content)
            .expect(&format!("Failed to deserialize from {}", config_path).as_str())
    } else {
        // If the file doesn't exist, create a new FlowProjectManagerConfig with default values.
        println!(
            "-> Could not read config file '{}'. Creating default config.",
            config_path
        );
        ServiceConfig::default()
    }
}

async fn handle_shutdown_signal(stopper: broadcast::Sender<()>) -> std::io::Result<()> {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to set up Ctrl+C handler");
    println!("-> Shutdown requested.");
    let _ = stopper.send(());
    Ok(())
}

#[tokio::main]
async fn main() {
    // Read Environment Variables
    dotenv().ok();
    let host_ip: String = std::env::var("HOST_IP").expect("HOST_IP must be set correctly");
    let host_ip_addr: IpAddr = IpAddr::V4(host_ip.parse::<Ipv4Addr>().unwrap());
    let host_port: String = std::env::var("HOST_PORT").expect("HOST_PORT must be set correctly");
    let host_port_u16: u16 = host_port.parse::<u16>().unwrap();

    let (stopper_sender, _) = broadcast::channel::<()>(1);
    let stopper_sender_clone = stopper_sender.clone();
    tokio::spawn(handle_shutdown_signal(stopper_sender_clone));

    let args = Arguments::parse();

    let config = load_config(&args.config_file.as_str());

    // Setup package manager
    let package_manager = Arc::new(Mutex::new(PackageManager::new_from_folder(
        &config.flow_packages_folder,
    )));

    // Setup project manager.
    let project_folder = config.flow_project_manager_config.project_folder.clone();
    let project_manager = Arc::new(Mutex::new(FlowProjectManager::new(
        config.flow_project_manager_config,
    )));
    let res = project_manager.lock().unwrap().load_projects();
    if let Err(err) = res {
        println!(
            "-> Failed to read project folder '{}'. Reason: {}",
            project_folder, err
        );
        println!("-> Create new project folder");
        if let Err(err) = fs::create_dir(&project_folder) {
            println!(
                "-> Failed to create new project folder '{}'. Reason: {}",
                project_folder, err
            );
            return;
        }
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let mut app = Router::new()
        .route("/packages/:package_name", get(get_package_by_name))
        .route("/packages/", get(get_all_packages))
        .with_state(package_manager.clone())
        //.route("/projects/:project_name", get(get_project_by_name))
        .route("/projects/", post(create_project))
        .with_state((project_manager.clone(), package_manager.clone()))
        .route("/projects/", get(get_all_projects))
        .route("/projects/:project_name/", delete(delete_project))
        .route("/projects/:project_name/compile", post(compile_project))
        .route(
            "/projects/:project_name/last_compile",
            get(last_compile_project),
        )
        .route("/projects/:project_name/run", post(run_project))
        .route("/processes/:process_id/stop", post(stop_process))
        .route("/processes/:process_id/logs", get(get_process_logs))
        .with_state(project_manager.clone());

    let addr = SocketAddr::new(host_ip_addr, host_port_u16);
    app = Router::new().nest("/api", app).layer(cors);

    println!("-> Listening on {}", addr);

    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            stopper_sender.subscribe().recv().await.ok();
        });

    // Run the server until it's gracefully shut down
    let _ = server.await;

    println!("-> Service shut down.");
}
