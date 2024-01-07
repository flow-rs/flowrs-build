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
    State(package_manager): State<Arc<Mutex<PackageManager>>>,
) -> Json<Vec<Package>> {
    let all_packages: Vec<Package> = package_manager.lock().unwrap().get_all_packages();
    Json(all_packages)
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
    use super::*;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use crate::{package::Package, package_manager::PackageManager};

    // Create mock data for PackageManager
    const MOCK_PACKAGE_NAME: &str = "test_package";
    const MOCK_PACKAGE_VERSION: &str = "1.0.0";

    // Function to create and return mock package Manager
    fn create_mock_pm() -> PackageManager {
        let mut mock_packages = HashMap::new();
        // Initialize MOCK_PACKAGE within the function
        let mock_package = Package {
            name: MOCK_PACKAGE_NAME.to_string(),
            version: MOCK_PACKAGE_VERSION.to_string(),
            crates: HashMap::new(),
        };
        mock_packages.insert(MOCK_PACKAGE_NAME.to_string(), mock_package);
        PackageManager {
            packages: mock_packages,
        }
    }

    #[tokio::test]
    async fn test_get_all_packages() {
        // Wrap PackageManager in Arc and Mutex
        let shared_package_manager = Arc::new(Mutex::new(create_mock_pm()));

        // Call the handler function
        let response = get_all_packages(State(shared_package_manager)).await;

        // Extract the response
        let packages = response.0;

        // Assert that the response contains the mock data
        assert_eq!(packages.len(), 1);
        assert!(packages.iter().any(|p| p.name == "test_package"));
    }

    #[tokio::test]
    async fn test_get_package_by_name() {
        // Wrap PackageManager in Arc and Mutex
        let shared_package_manager = Arc::new(Mutex::new(create_mock_pm()));

        // Call the handler function
        let response = get_package_by_name(
            axum::extract::Path(MOCK_PACKAGE_NAME.to_string()),
            State(shared_package_manager),
        )
        .await;

        // Extract the response
        let package = response.unwrap().0.unwrap();

        // Assert that the response contains the mock data
        assert_eq!(package.name, MOCK_PACKAGE_NAME);
        assert_eq!(package.version, MOCK_PACKAGE_VERSION);
    }

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
