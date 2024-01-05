use std::{fs::File, io::Read};

use serde::{Deserialize, Serialize};

use crate::flow_project::FlowProjectManagerConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    #[serde(default = "flow_project_manager_config_default")]
    pub flow_project_manager_config: FlowProjectManagerConfig,

    #[serde(default = "flow_packages_folder_default")]
    pub flow_packages_folder: String,
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

pub fn load_config(config_path: &str) -> ServiceConfig {
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
