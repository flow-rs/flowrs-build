use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{self, ErrorKind};
use std::{fs, path::Path};
use toml::{self, Value};

use cargo_metadata::{MetadataCommand, Package};

use syn::{Item, ItemMod};

use flowrs_package::flow_package::package::Crate as FlowCrate;
use flowrs_package::flow_package::package::Package as FlowPackage;
//use flowrs_package::flow_package::package_manager::PackageManager as FlowPackageManager;

const DEBUG: &str = "cargo::warning= [DEBUG]: ";

// Function to determine if a Cargo Package is a flow-package
fn is_flow_package(package: Package) -> bool {
    package.keywords.contains(&"flow-package".to_string())
}

// Crates new FlowPackage and adds extracted name and version from crate package
fn extract_flow_package_name_and_version(package_path: &Path) -> Result<FlowPackage, io::Error> {
    // Read Cargo.toml into HashMap
    let file_path = package_path.join("Cargo.toml");
    let file_content: String = read_to_string(file_path)?;
    let data: Value = file_content.parse()?;

    // Read name and version values
    println!("{}{:?}", DEBUG, data);
    let package_section = data
        .get("package")
        .ok_or(io::Error::from(ErrorKind::NotFound))
        .unwrap();
    let name = package_section
        .get("name")
        .and_then(Value::as_str)
        .ok_or(io::Error::from(ErrorKind::NotFound))
        .unwrap();
    let version = package_section
        .get("version")
        .and_then(Value::as_str)
        .ok_or(io::Error::from(ErrorKind::NotFound))
        .unwrap();

    // Set name and version values and return flow package
    let mut flow_package = FlowPackage::default();
    flow_package.name = name.to_string();
    flow_package.version = version.to_string();

    Ok(flow_package)
}

// Function to extract flow-crates from cargo-package
fn extract_flow_crates(_cargo_package: Package, package_path: &Path) -> HashMap<String, FlowCrate> {
    let src_path = package_path.join("src");
    //let nodes_path = src_path.join("nodes");
    let lib_path = src_path.join("lib.rs");

    // Parse lib.rs to get the module structure
    let file_content = fs::read_to_string(lib_path).expect("Unable to read lib.rs");
    let syntax_tree = syn::parse_file(&file_content).expect("Unable to parse lib.rs file content");
    for item in syntax_tree.items {
        match item {
            Item::Mod(m) => parse_module(m),
            Item::Const(_) => (),
            Item::Enum(_) => (),
            Item::ExternCrate(_) => (),
            Item::Fn(_) => (),
            Item::ForeignMod(_) => (),
            Item::Impl(_) => (),
            Item::Macro(_) => (),
            Item::Static(_) => (),
            Item::Struct(_) => (),
            Item::Trait(_) => (),
            Item::TraitAlias(_) => (),
            Item::Type(_) => (),
            Item::Union(_) => (),
            Item::Use(_) => (),
            Item::Verbatim(_) => (),
            _ => (),
        }
    }

    HashMap::new()
}

fn parse_module(_module: ItemMod) {}

fn main() {
    let debug: bool = true;

    // Fetch Metadata from the cargo.toml
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to fetch cargo metadata");

    for crate_package in metadata.packages {
        // Filter for dependencies containing nodes
        if is_flow_package(crate_package.clone()) {
            let package_path = Path::new(&crate_package.manifest_path).parent().unwrap();
            if debug {
                println!("{}PACKAGE-PATH={}", DEBUG, package_path.to_str().unwrap(),);
            }

            // Extract Node and Flow-Package information
            let mut flow_package: FlowPackage =
                extract_flow_package_name_and_version(package_path).unwrap();
            flow_package.crates = extract_flow_crates(crate_package.clone(), package_path);
        }
    }
}
