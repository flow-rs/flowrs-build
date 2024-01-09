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
        fs::create_dir,
        sync::{Arc, Mutex},
    };

    use crate::{
        flow_model::FlowModel,
        flow_project::{
            builtin_dependencies_default, delete_folder_recursive, do_formatting_default,
            project_json_file_name_default, rust_fmt_path_default, FlowProjectManagerConfig,
            Process,
        },
        package::Package,
        package_manager::PackageManager,
    };
    use tokio::time::{sleep, Duration};

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

    const TEST_PROJECT_FOLDER: &str = "flow-projects-test-handlers";
    const TEST_PROJECT_NAME: &str = "test-project";
    const TEST_PROJECT_NAME2: &str = "test-project2";
    const TEST_PROJECT_VERSION: &str = "1.0.0";

    fn get_project_json() -> String {
        return "{\n  \"name\": \"".to_string()
            + TEST_PROJECT_NAME
            + "\",\n  \"version\": \""
            + TEST_PROJECT_VERSION
            + "\",\n"
            + r#"
  "packages": [
    {"name": "flowrs", "version": "1.0.0", "git":"https://github.com/flow-rs/flowrs", "branch":"feature-project7"},
    {"name": "flowrs-std", "version": "1.0.0", "git":"https://github.com/flow-rs/flowrs-std", "branch":"feature-project1"}
    ],
  "flow":{        
        "nodes": {
            "debug_node": {
                "node_type": "flowrs_std::nodes::debug::DebugNode",
                "type_parameters": {"I": "i32"},
                "constructor": "New"

            },
            "timer_config_node": {
                "node_type": "flowrs_std::nodes::value::ValueNode",
                "type_parameters": {"I": "flowrs_std::nodes::timer::TimerNodeConfig"},
                "constructor": "New"

            },
            "timer_token_node": {
                "node_type": "flowrs_std::nodes::value::ValueNode",
                "type_parameters": {"I": "i32"},
                "constructor": "New"

            },
             "timer_node": {
                "node_type": "flowrs_std::nodes::timer::TimerNode",
                "type_parameters": {"T": "flowrs_std::nodes::timer::SelectedTimer", "U": "i32"},
                "constructor": "New"
            }
        },
        "connections": [
            {
                "from_node": "timer_config_node",
                "from_output": "output",
                "to_node": "timer_node",
                "to_input": "config_input"
            },
            {
                "from_node": "timer_token_node",
                "from_output": "output",
                "to_node": "timer_node",
                "to_input": "token_input"
            },
            {
                "from_node": "timer_node",
                "from_output": "token_output",
                "to_node": "debug_node",
                "to_input": "input"
            }
        ], 
        "data" : {
            "timer_config_node": {
                "value": {"duration": {"secs": 1, "nanos": 0}}
            },
             "timer_token_node": {
                "value": 42
            }
        }
    }
}
    "#;
    }

    // Function to create and return a test FlowProjectManager
    fn create_test_fpm() -> FlowProjectManager {
        let _ = create_dir(std::path::Path::new(&TEST_PROJECT_FOLDER));
        let config = FlowProjectManagerConfig {
            project_folder: TEST_PROJECT_FOLDER.to_string(),
            project_json_file_name: project_json_file_name_default(),
            builtin_dependencies: builtin_dependencies_default(),
            rust_fmt_path: rust_fmt_path_default(),
            do_formatting: do_formatting_default(),
        };

        // Initialize the FlowProjectManager with the test configuration
        let flow_project_manager = FlowProjectManager::new(config);

        flow_project_manager
    }

    fn add_test_project(mut fpm: FlowProjectManager) -> FlowProjectManager {
        let flow_project = FlowProject {
            name: TEST_PROJECT_NAME2.to_string(),
            version: TEST_PROJECT_VERSION.to_string(),
            packages: vec![],
            flow: FlowModel {
                nodes: HashMap::new(),
                connections: vec![],
                data: "".into(),
            },
        };

        let _ = fpm.create_flow_project(flow_project, &create_mock_pm());
        fpm
    }

    fn cleanup_flow_projects() {
        let _ = delete_folder_recursive(std::path::Path::new(&TEST_PROJECT_FOLDER));
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
    async fn test_get_all_projects() {
        // Create test fpm
        let mut fpm = create_test_fpm();

        // Add test project
        fpm = add_test_project(fpm);

        // Wrap FlowPackageManager in Arc and Mutex
        let shared_fpm = Arc::new(Mutex::new(fpm));

        // Call the handler function
        let flow_projects = get_all_projects(State(shared_fpm)).await.0;

        //Cleanup
        cleanup_flow_projects();

        // Extract the response
        assert_eq!(flow_projects.len(), 1);
        assert!(flow_projects.iter().any(|fp| fp.name == TEST_PROJECT_NAME2));
        assert!(flow_projects
            .iter()
            .any(|fp| fp.version == TEST_PROJECT_VERSION));
    }

    #[tokio::test]
    async fn test_project_handlers() {
        // Create test FlowProjectManager and PackageManager
        let fpm = create_test_fpm();
        let pm = PackageManager::new_from_folder("flow-packages");

        // Wrap FlowPackageManager and PackageManager in Arc and Mutex
        let shared_fpm = Arc::new(Mutex::new(fpm));
        let shared_pm = Arc::new(Mutex::new(pm));
        test_create_project(shared_fpm.clone(), shared_pm).await;
        sleep(Duration::from_secs(100)).await;
        test_compile_project(shared_fpm.clone()).await;
        sleep(Duration::from_secs(10)).await;
        let process_id: String = test_run_project(shared_fpm.clone()).await;
        sleep(Duration::from_secs(10)).await;
        test_get_process_logs(shared_fpm.clone(), process_id.clone()).await;
        test_stop_project(shared_fpm.clone(), process_id).await;
        test_delete_project(shared_fpm.clone()).await;

        //Cleanup
        cleanup_flow_projects();
    }

    async fn test_create_project(
        shared_fpm: Arc<Mutex<FlowProjectManager>>,
        shared_pm: Arc<Mutex<PackageManager>>,
    ) {
        let flow_project_res = serde_json::from_str(&get_project_json());
        assert!(!flow_project_res.is_err());
        let flow_project: FlowProject = flow_project_res.unwrap();

        // Call the handler function
        let response = create_project(State((shared_fpm, shared_pm)), Json(flow_project))
            .await
            .unwrap();

        // Check the response status
        assert_eq!(response.status(), StatusCode::CREATED);

        // Collect the body stream into complete bytes
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        // Convert the bytes to a String
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Deserialize the response body to FlowProject
        let created_project: FlowProject = serde_json::from_str(&body_string).unwrap();

        // Assert that the created project matches the test project
        assert_eq!(created_project.name, TEST_PROJECT_NAME);
        assert_eq!(created_project.version, TEST_PROJECT_VERSION);
    }

    async fn test_delete_project(shared_fpm: Arc<Mutex<FlowProjectManager>>) {
        // Call the delete_project handler function
        let response = delete_project(
            axum::extract::Path(TEST_PROJECT_NAME.to_string()),
            State(shared_fpm.clone()),
        )
        .await
        .unwrap();

        // Check the response status
        assert_eq!(response.status(), StatusCode::OK);

        // Verify that the project is indeed deleted from the project manager
        let fpm_locked = shared_fpm.lock().unwrap();
        assert!(!fpm_locked.projects.contains_key(TEST_PROJECT_NAME));
    }

    async fn test_compile_project(shared_fpm: Arc<Mutex<FlowProjectManager>>) {
        // Define build type for compilation
        let build_type = BuildType {
            build_type: "cargo".to_string(),
        };

        // Call the handler function
        let response = compile_project(
            axum::extract::Path(TEST_PROJECT_NAME.to_string()),
            State(shared_fpm.clone()),
            Query(build_type),
        )
        .await
        .unwrap();

        // Check the response status
        assert_eq!(response.status(), StatusCode::OK);
    }

    async fn test_run_project(shared_fpm: Arc<Mutex<FlowProjectManager>>) -> String {
        // Define build type for running the project
        let build_type = BuildType {
            build_type: "cargo".to_string(), // Assuming cargo build type for this test
        };

        let response = run_project(
            axum::extract::Path(TEST_PROJECT_NAME.to_string()),
            State(shared_fpm.clone()),
            Query(build_type),
        )
        .await
        .unwrap();

        // Check the response status
        assert_eq!(response.status(), StatusCode::CREATED);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
        let running_process: Process = serde_json::from_str(&body_string).unwrap();

        running_process.process_id.to_string()
    }

    async fn test_stop_project(shared_fpm: Arc<Mutex<FlowProjectManager>>, process_id: String) {
        // Call the stop_process handler function with the process ID
        let response = stop_process(axum::extract::Path(process_id), State(shared_fpm))
            .await
            .unwrap();

        // Check the response status
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    async fn test_get_process_logs(shared_fpm: Arc<Mutex<FlowProjectManager>>, process_id: String) {
        // Call the get_process_logs handler function with the process ID
        let response = get_process_logs(axum::extract::Path(process_id), State(shared_fpm.clone()))
            .await
            .unwrap();

        // Check the response status
        assert_eq!(response.status(), StatusCode::CREATED);

        // Optionally, deserialize the response body to verify the logs
        let logs_body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let logs_body_string = String::from_utf8(logs_body_bytes.to_vec()).unwrap();
        let logs: Vec<String> = serde_json::from_str(&logs_body_string).unwrap();

        // Assert that logs are retrieved (or assert on specific log content if expected)
        assert!(!logs.is_empty());
    }

    //async fn test_last_compile_project() {}
}
