use std::{
    fs::File,
    io::Read,
    net::{IpAddr, Ipv4Addr},
};

use serde::{Deserialize, Serialize};

use crate::flow_project::FlowProjectManagerConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowrsConfig {
    #[serde(default = "flow_project_manager_config_default")]
    pub flow_project_manager_config: FlowProjectManagerConfig,

    #[serde(default = "flow_packages_folder_default")]
    pub flow_packages_folder: String,
}

impl Default for FlowrsConfig {
    fn default() -> Self {
        Self {
            flow_project_manager_config: flow_project_manager_config_default(),
            flow_packages_folder: flow_packages_folder_default(),
        }
    }
}

fn flow_project_manager_config_default() -> FlowProjectManagerConfig {
    FlowProjectManagerConfig::default()
}

fn flow_packages_folder_default() -> String {
    "flow-packages".to_string()
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub flowrs: FlowrsConfig,
    pub ip: IpAddr,
    pub port: u16,
}

impl ServiceConfig {
    pub fn new(flowrs_config: FlowrsConfig, ip: IpAddr, port: u16) -> Self {
        Self {
            flowrs: flowrs_config,
            ip: ip,
            port: port,
        }
    }
}

fn load_config(config_path: &str) -> FlowrsConfig {
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
        FlowrsConfig::default()
    }
}

pub fn load_environment_and_configure() -> ServiceConfig {
    // Read Environment Variables
    dotenv::dotenv().ok();
    let host_ip: String = std::env::var("HOST_IP").expect("HOST_IP must be set correctly");
    let host_ip_addr: IpAddr = IpAddr::V4(host_ip.parse::<Ipv4Addr>().unwrap());
    let host_port: String = std::env::var("HOST_PORT").expect("HOST_PORT must be set correctly");
    let host_port_u16: u16 = host_port.parse::<u16>().unwrap();
    let config_path: String =
        std::env::var("CONFIG_PATH").expect("CONFIG_PATH must be set correctly");

    let flowrs_config = load_config(&config_path);
    let service_config = ServiceConfig::new(flowrs_config, host_ip_addr, host_port_u16);

    service_config
}
