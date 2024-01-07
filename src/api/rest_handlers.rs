use crate::flow_project::{BuildType, FlowProject, FlowProjectManager};
use crate::package::Package;
use crate::package_manager::PackageManager;

use std::sync::{Arc, Mutex};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{Response, StatusCode},
    Json,
};

pub async fn get_all_packages(
    State(package_manager): State<Arc<Mutex<dyn PackageManagerTrait>>>,
) -> Json<Vec<Package>> {
    Json(package_manager.lock().unwrap().get_all_packages())
}

pub async fn get_package_by_name(
    Path(package_name): Path<String>,
    State(package_manager): State<Arc<Mutex<PackageManager>>>,
) -> Result<Json<Option<Package>>, StatusCode> {
    if let Some(package) = package_manager.lock().unwrap().get_package(&package_name) {
        return Ok(Json(Some(package.clone())));
    }

    Err(StatusCode::NOT_FOUND)
}

pub async fn get_all_projects(
    State(project_manager): State<Arc<Mutex<FlowProjectManager>>>,
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

pub async fn create_project(
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

pub async fn delete_project(
    Path(project_name): Path<String>,
    State(project_manager): State<Arc<Mutex<FlowProjectManager>>>,
) -> Result<Response<Body>, StatusCode> {
    match project_manager
        .lock()
        .unwrap()
        .delete_flow_project(project_name.as_str())
    {
        Ok(result) => {
            // Return a success response with the created object in the body
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(result))
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

pub async fn compile_project(
    Path(project_name): Path<String>,
    State(project_manager): State<Arc<Mutex<FlowProjectManager>>>,
    build_type: Query<BuildType>,
) -> Result<Response<Body>, StatusCode> {
    match project_manager
        .lock()
        .unwrap()
        .compile_flow_project(project_name.as_str(), build_type.0.build_type)
    {
        Ok(result) => {
            // Return a success response with the created object in the body
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(result))
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

pub async fn last_compile_project(
    Path(project_name): Path<String>,
    State(project_manager): State<Arc<Mutex<FlowProjectManager>>>,
    build_type: Query<BuildType>,
) -> Result<Response<Body>, StatusCode> {
    match project_manager
        .lock()
        .unwrap()
        .last_compile_flow_project(project_name.as_str(), build_type.0.build_type)
    {
        Ok(result) => {
            // Return a success response with the created object in the body
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(result))
                .unwrap();

            Ok(response)
        }
        Err(err) => {
            // Return an error response with status code and error message in the body
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(err.to_string()))
                .unwrap();

            Ok(response)
        }
    }
}

pub async fn run_project(
    Path(project_name): Path<String>,
    State(project_manager): State<Arc<Mutex<FlowProjectManager>>>,
    build_type: Query<BuildType>,
) -> Result<Response<Body>, StatusCode> {
    match project_manager
        .lock()
        .unwrap()
        .run_flow_project(project_name.as_str(), build_type.0.build_type)
    {
        Ok(process) => {
            // Return a success response with the created object in the body
            let response = Response::builder()
                .status(StatusCode::CREATED)
                .body(Body::from(serde_json::to_string(&process).unwrap()))
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

pub async fn stop_process(
    Path(process_id): Path<String>,
    State(project_manager): State<Arc<Mutex<FlowProjectManager>>>,
) -> Result<Response<Body>, StatusCode> {
    match project_manager.lock().unwrap().stop_process(process_id) {
        Ok(result) => {
            // Return a success response with the created object in the body
            let response = Response::builder()
                .status(StatusCode::CREATED)
                .body(Body::from(result))
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

pub async fn get_process_logs(
    Path(process_id): Path<String>,
    State(project_manager): State<Arc<Mutex<FlowProjectManager>>>,
) -> Result<Response<Body>, StatusCode> {
    match project_manager.lock().unwrap().get_process_logs(process_id) {
        Ok(result) => {
            // Return a success response with the created object in the body
            let response = Response::builder()
                .status(StatusCode::CREATED)
                .body(Body::from(serde_json::to_string(&result).unwrap()))
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

#[cfg(test)]
mod tests {
    use crate::package_manager::MockPackageManagerTrait;

    use super::*;
    use axum::extract::State;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    #[tokio::test]
    async fn test_get_all_packages() {
        let mut mock_package_manager = MockPackageManagerTrait::new();

        // Setup mock behavior
        let expected_packages = vec![Package {
            name: "testpackage".to_string(),
            version: "testversion".to_string(),
            crates: HashMap::new(),
        }]; // Define expected packages
        mock_package_manager
            .expect_get_all_packages()
            .return_const(expected_packages.clone());

        let package_manager = Arc::new(Mutex::new(Box::new(mock_package_manager)));

        // Call your handler function
        let response = get_all_packages(State(package_manager)).await;

        // Assert the response
        assert_eq!(response, Json(expected_packages));
    }

    #[tokio::test]
    async fn test_get_package_by_name() {}

    #[tokio::test]
    async fn test_get_all_projects() {}

    #[tokio::test]
    async fn test_create_project() {}

    #[tokio::test]
    async fn test_delete_project() {}

    #[tokio::test]
    async fn test_compile_project() {}

    #[tokio::test]
    async fn test_last_compile_project() {}

    #[tokio::test]
    async fn test_run_project() {}

    #[tokio::test]
    async fn test_stop_project() {}

    #[tokio::test]
    async fn test_get_process_logs() {}
}
