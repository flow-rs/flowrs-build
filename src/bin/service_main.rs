use axum::{
    body::{Body, StreamBody},
    extract::{Path, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tokio_util::io::ReaderStream;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::{path::{PathBuf}, process::Command};
use std::fs::File;
use std::io::Read;


use flowrs_build::{
    flow_project::{FlowProject, FlowProjectManager, FlowProjectManagerConfig},
    package::Package,
    package_manager::PackageManager,
};
use serde::{Deserialize, Serialize};

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
            flow_packages_folder: flow_packages_folder_default()
        }
    }
}

fn load_config(config_path: &str) -> ServiceConfig {
    
    if std::path::Path::new(config_path).exists() {
        // Read and deserialize the existing config.json file
        let mut file = File::open(config_path).expect("Failed to open config.json");
        let mut config_content = String::new();
        file.read_to_string(&mut config_content)
            .expect("Failed to read config.json");
        serde_json::from_str(&config_content).expect("Failed to deserialize config.json")
    } else {
        // If the file doesn't exist, create a new FlowProjectManagerConfig with default values
        ServiceConfig::default()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = load_config("config.json");

    let package_manager = Arc::new(Mutex::new(PackageManager::new_from_folder(&config.flow_packages_folder)));

    let project_manager = Arc::new(Mutex::new(FlowProjectManager::new(config.flow_project_manager_config)));

    let res = project_manager.lock().unwrap().load_projects();
    if let Err(err) = res {
        eprintln!("Could not load projects. Reason: {}", err);
        return;
    }

    let app = Router::new()
        .route("/build/:project_name", get(build_package))
        .route("/file/:project_name/:file_name", get(get_file))
        .with_state(project_manager.clone())
        .route("/packages/:package_name", get(get_package_by_name))
        .route("/packages/", get(get_all_packages))
        .with_state(package_manager.clone())
        //.route("/projects/:project_name", get(get_project_by_name))
        .route("/projects/", get(get_all_projects))
        .route("/projects/", post(create_project))
        .with_state((project_manager.clone(), package_manager.clone()));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_file(
    Path(project_name): Path<String>,
    Path(file_name): Path<String>,
    State(project_manager): State<Arc<Mutex<FlowProjectManager>>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let project = project_manager
        .lock()
        .unwrap()
        .projects
        .get(&project_name)
        .cloned();
    if let None = project {
        let error_message = format!(
            "The Project {} does not exist in the Project Manager",
            project_name
        );
        eprintln!("{}", error_message);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, error_message.to_string()));
    }
    let project_path = PathBuf::from(format!("./flow-projects/{}", project_name));
    let target_dir = project_path.join("pkg");

    if !target_dir.exists() {
        let error_message = "The target dir of the generated WASM file cannot be found.";
        eprintln!("{}", error_message);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, error_message.to_string()));
    }

    let file_name = file_name;
    let file_path = target_dir.join(file_name.clone());
    let file = tokio::fs::File::open(file_path.clone())
        .await
        .expect("Failed to open Wasm file");
    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);
    let content_type = match mime_guess::from_path(file_path).first_raw() {
        Some(mime) => mime,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "MIME Type couldn't be determined".to_string(),
            ))
        }
    };
    let headers = [
        (header::CONTENT_TYPE, content_type.to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{:?}\"", file_name),
        ),
    ];

    Ok((headers, body))
}

async fn build_package(
    State(project_manager): State<Arc<Mutex<FlowProjectManager>>>,
    Path(project_name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let project = project_manager
        .lock()
        .unwrap()
        .projects
        .get(&project_name)
        .cloned();

    if let None = project {
        let error_message = format!(
            "The Project {} does not exist in the Project Manager",
            project_name
        );
        eprintln!("{}", error_message);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, error_message.to_string()));
    }

    let project_path = PathBuf::from(format!("./flow-projects/{}", project_name));

    if !project_path.exists() {
        let error_message = "The specified project directory does not exist.";
        eprintln!("{}", error_message);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, error_message.to_string()));
    }

    if let Err(err) = std::env::set_current_dir(&project_path) {
        let error_message = format!("Failed to change the working directory: {}", err);
        eprintln!("{} {}", error_message, err);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, error_message.to_string()));
    }

    let wasm_pack_build = Command::new("wasm-pack")
        .args(&["build", "--target", "web"])
        .output();

    if let Err(err) = wasm_pack_build {
        let error_message = format!("Couldn't build the project: {}", err);
        eprintln!("{} {}", error_message, err);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, error_message.to_string()));
    }

    let output = wasm_pack_build.unwrap();

    if !output.status.success() {
        eprintln!("Failed to build the project for WebAssembly.");
        if let Some(stderr) = String::from_utf8_lossy(&output.stderr)
            .trim()
            .to_string()
            .split('\n')
            .next()
        {
            eprintln!("Error: {}", stderr);
        }
    }

    println!("Project built successfully!");
    let target_dir = project_path.join("pkg");

    if !target_dir.exists() {
        let error_message = "The target dir of the generated WASM file cannot be found.";
        eprintln!("{}", error_message);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, error_message.to_string()));
    }

    let file_name = format!("{}_bg.wasm", project_name);
    let wasm_file_path = target_dir.join(file_name.clone());
    let wasm_file = tokio::fs::File::open(wasm_file_path)
        .await
        .expect("Failed to open Wasm file");
    let stream = ReaderStream::new(wasm_file);
    let body = StreamBody::new(stream);
    let headers = [
        (header::CONTENT_TYPE, "application/wasm".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{:?}\"", file_name),
        ),
    ];

    Ok((headers, body))
}

async fn get_all_packages(
    State(package_manager): State<Arc<Mutex<PackageManager>>>,
) -> Json<Vec<Package>> {
    Json(package_manager.lock().unwrap().get_all_packages())
}

async fn get_package_by_name(
    Path(package_name): Path<String>,
    State(package_manager): State<Arc<Mutex<PackageManager>>>,
) -> Result<Json<Option<Package>>, StatusCode> {
    if let Some(package) = package_manager.lock().unwrap().get_package(&package_name) {
        return Ok(Json(Some(package.clone())));
    }

    Err(StatusCode::NOT_FOUND)
}

async fn get_all_projects(
    State((project_manager, package_manager)): State<(
        Arc<Mutex<FlowProjectManager>>,
        Arc<Mutex<PackageManager>>,
    )>,
) -> Json<Vec<FlowProject>> {
    let all_projects: Vec<FlowProject> = project_manager
        .lock()
        .unwrap()
        .projects
        .values()
        .cloned()
        .collect();
    Json(all_projects)
}

async fn create_project(
    State((project_manager, package_manager)): State<(
        Arc<Mutex<FlowProjectManager>>,
        Arc<Mutex<PackageManager>>,
    )>,
    Json(flow_project): Json<FlowProject>,
) -> Result<Response<Body>, StatusCode> {
    match project_manager
        .lock()
        .unwrap()
        .create_flow_project(flow_project, &package_manager.lock().unwrap())
    {
        Ok(flow_project) => {
            // Return a success response with the created object in the body
            let response = Response::builder()
                .status(StatusCode::CREATED)
                .body(Body::from(serde_json::to_string(&flow_project).unwrap()))
                .unwrap();

            Ok(response)
        }
        Err(err) => {
            // Return an error response with status code and error message in the body
            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(err.to_string()))
                .unwrap();

            Ok(response)
        }
    }
}
