use axum::{
    routing::{get, post},
    body::Body,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
    Json, Router,
    extract::{Path, Extension, State}
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::collections::HashMap;

use flowrs_build::{package::{Package, load_packages}, flow_project::{FlowProject, FlowProjectManager}};

#[tokio::main]
async fn main() {
    
    tracing_subscriber::fmt::init();

    let packages = load_packages("packages");

    let mut project_manager = FlowProjectManager::new("flow-projects");
    let res = project_manager.load_projects();
    if let Err(err) = res {
        eprintln!("Could not load projects. Reason: {}", err);
        return;
    }

    let app = Router::new()
        
        .route("/packages/:package_name", get(get_package_by_name))
        .route("/packages/", get(get_all_packages))
        .with_state(packages)

        //.route("/projects/:project_name", get(get_project_by_name))
        .route("/projects/", get(get_all_projects))
        .route("/projects/", post(create_project))
        .with_state(project_manager);


       
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_all_packages(State(packages): State<HashMap<String, Package>>) -> Json<Vec<Package>> {
    let all_packages: Vec<Package> = packages.values().cloned().collect();
    Json(all_packages)
}

async fn get_package_by_name(
    Path(package_name): Path<String>,
    State(packages): State<HashMap<String, Package>>,
) -> Result<Json<Option<Package>>, StatusCode> {

    if let Some(package) = packages.get(&package_name) {
        return Ok(Json(Some(package.clone())));
    }

    Err(StatusCode::NOT_FOUND)
}

async fn get_all_projects(State(project_manager): State<FlowProjectManager>) -> Json<Vec<FlowProject>> {
    let all_projects: Vec<FlowProject> = project_manager.projects.values().cloned().collect();
    Json(all_projects)
}

async fn create_project(
    State(mut project_manager): State<FlowProjectManager>,
    Json(flow_project): Json<FlowProject>,
) -> Result<Response<Body>, StatusCode> {
    match project_manager.create_flow_project(flow_project) {
        Ok(flow_project) => {
            // Return a success response with the created object in the body
            let response = Response::builder()
                .status(StatusCode::CREATED)
                .body(Body::from(serde_json::to_string(&flow_project).unwrap()))
                .unwrap();

            Ok(response)
        },
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