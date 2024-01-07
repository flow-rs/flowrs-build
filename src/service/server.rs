use std::{
    fs,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Error};
use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::{
    api::rest_handlers::{
        compile_project, create_project, delete_project, get_all_packages, get_all_projects,
        get_package_by_name, get_process_logs, last_compile_project, run_project, stop_process,
    },
    flow_project::FlowProjectManager,
    package_manager::PackageManager,
};

use super::config::ServiceConfig;

pub fn setup_server(server_config: ServiceConfig) -> Result<Router, Error> {
    let config = server_config.flowrs;
    // Setup package manager
    let package_manager = Arc::new(Mutex::new(PackageManager::new_from_folder(
        &config.flow_packages_folder,
    )));

    // Setup project manager.
    let project_folder = config.flow_project_manager_config.project_folder.clone();
    let project_manager = Arc::new(Mutex::new(FlowProjectManager::new(
        config.flow_project_manager_config,
    )));

    project_manager
        .lock()
        .unwrap()
        .load_projects()
        .map_err(|err| {
            anyhow!(
                "-> Failed to read project folder '{}'. Reason: {}",
                project_folder,
                err
            )
        })?;

    fs::create_dir(&project_folder).map_err(|err| {
        anyhow!(
            "Failed to create new project folder '{}': {}",
            project_folder,
            err
        )
    })?;

    let app = Router::new()
        .route("/packages/:package_name", get(get_package_by_name))
        //.with_state(package_manager.clone())
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
        .with_state(project_manager.clone())
        .layer(CorsLayer::permissive());

    Ok(Router::new().nest("/api", app))
}
