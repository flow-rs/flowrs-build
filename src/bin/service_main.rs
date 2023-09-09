use axum::{
    body::{Body, StreamBody},
    extract::{Path, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tokio_util::io::ReaderStream;

use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::{path::PathBuf, process::Command};

use flowrs_build::{
    flow_project::{FlowProject, FlowProjectManager},
    package::Package,
    package_manager::PackageManager,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let package_manager = Arc::new(Mutex::new(PackageManager::new_from_folder("packages")));

    let project_manager = Arc::new(Mutex::new(FlowProjectManager::new("flow-projects")));

    let res = project_manager.lock().unwrap().load_projects();
    if let Err(err) = res {
        eprintln!("Could not load projects. Reason: {}", err);
        return;
    }

    let app = Router::new()
        .route("/packages/:package_name", get(get_package_by_name))
        .route("/build/:package_name", get(build_package))
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

async fn build_package(
    State(package_manager): State<Arc<Mutex<PackageManager>>>,
    Path(package_name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let package = package_manager
        .lock()
        .unwrap()
        .get_package(&package_name)
        .cloned()
        .unwrap();
    let project_path = PathBuf::from(format!("./flow-projects/{}", package.name));

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

    let file_name = format!("{}_bg.wasm", package.name);
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
