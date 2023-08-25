use serde::{Deserialize, Serialize};
use serde_json;

use std::collections::HashMap;
use std::fs;

use crate::version::Version;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Package {
    name: String,
    version: String,
    crates: HashMap<String, Crate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Crate {
    nodes: HashMap<String, NodeType>,
    types: HashMap<String, ArbitraryType>,
    modules: HashMap<String, Module>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Module {
    nodes: HashMap<String, NodeType>,
    types: HashMap<String, ArbitraryType>,
    modules: HashMap<String, Module>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NodeType {
    inputs: Vec<String>,
    outputs: Vec<String>,
    type_parameters: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ArbitraryType;

pub fn load_packages(directory_path: &str) -> HashMap<String, Package> {
    let mut packages: HashMap<String, Package> = HashMap::new();

    if let Ok(entries) = fs::read_dir(directory_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(extension) = entry.path().extension() {
                    if extension == "json" {
                        if let Some(file_name) = entry.path().file_stem() {
                            let package_name = file_name.to_string_lossy().to_string();

                            if let Ok(contents) = fs::read_to_string(entry.path()) {
                                
                                let package = serde_json::from_str::<Package>(&contents);
                                match package {
                                    Ok(p) => {
                                        packages.insert(package_name.clone(), p);
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Failed to deserialize package: {}. Reason: {}",
                                            package_name,
                                            e.to_string()
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    packages
}
