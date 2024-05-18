use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::{fs, path::Path};
use toml::{self, Value};

use cargo_metadata::{MetadataCommand, Package};

use syn::{Item, ItemMod, ItemType};

use flowrs_package::flow_package::package::Crate as FlowCrate;
use flowrs_package::flow_package::package::Module as FlowModule;
use flowrs_package::flow_package::package::Package as FlowPackage;
use flowrs_package::flow_package::package::Type as FlowType;
//use flowrs_package::flow_package::package_manager::PackageManager as FlowPackageManager;

const DEBUG_STR: &str = "cargo::warning= [DEBUG]:";

fn debug(message: String) {
    let debug = true;

    if debug {
        println!("{} {}", DEBUG_STR, message)
    }
}

pub struct FlowModuleWrapper {
    pub name: String,
    pub flow_module: FlowModule,
}

pub struct FlowTypeWrapper {
    pub name: String,
    pub flow_type: FlowType,
}

// Function to determine if a Cargo Package is a flow-package
fn is_flow_package(package: Package) -> bool {
    package.keywords.contains(&"flow-package".to_string())
}

// Crates new FlowPackage and adds extracted name and version from crate package
fn extract_flow_package_name_and_version(package_path: &Path) -> Result<FlowPackage, io::Error> {
    // Read Cargo.toml
    let file_path = package_path.join("Cargo.toml");
    let file_content: String = fs::read_to_string(file_path)?;
    let data: Value = file_content.parse()?;

    // Read name and version values
    debug(format!("{:?}", data));
    let package_section = data
        .get("package")
        .ok_or(io::Error::from(ErrorKind::InvalidData))
        .unwrap();
    let name = package_section
        .get("name")
        .and_then(Value::as_str)
        .ok_or(io::Error::from(ErrorKind::InvalidData))
        .unwrap();
    let version = package_section
        .get("version")
        .and_then(Value::as_str)
        .ok_or(io::Error::from(ErrorKind::InvalidData))
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
    let lib_tree = syn::parse_file(&file_content).expect("Unable to parse lib.rs file content");
    let mut crates: HashMap<String, FlowCrate> = HashMap::new();
    let mut types: HashMap<String, FlowType> = HashMap::new();
    let mut modules: HashMap<String, FlowModule> = HashMap::new();
    for item in lib_tree.items {
        match item {
            Item::Mod(m) => {
                let module_wrapper = parse_module(m, package_path);
                modules.insert(module_wrapper.name, module_wrapper.flow_module);
            }
            Item::Type(t) => {
                let type_wrapper = parse_type(t);
                types.insert(type_wrapper.name, type_wrapper.flow_type);
            }
            _ => (), // Non-Mod Items are not relevant
        }
    }

    HashMap::new()
}

fn parse_module(module: ItemMod, path: &Path) -> FlowModuleWrapper {
    let module_name = module.ident.to_string();
    let module_file_path = path.join(module_name.clone()).with_extension("rs");
    // try to parse module file
    let file_content = match fs::read_to_string(module_file_path) {
        Ok(file_content) => file_content,
        Err(_) => return,
    };
    let module_tree = syn::parse_file(&file_content)
        .expect(format!("Unable to parse {}.rs file content", module_name.clone()).as_str());
    for item in module_tree.items {
        match item {
            Item::Mod(m) => parse_module(m, &path.join(module_name.clone())),
            _ => (), // Non-Mod Items are not relevant
        }
    }
    //debug(format!("{:?}", module_file_path));
}

fn parse_type(flow_type: ItemType) -> FlowTypeWrapper {
    todo!();
}

fn main() {
    // Fetch Metadata from the cargo.toml
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to fetch cargo metadata");

    for crate_package in metadata.packages {
        // Filter for dependencies containing nodes
        if is_flow_package(crate_package.clone()) {
            let package_path = Path::new(&crate_package.manifest_path).parent().unwrap();
            debug(package_path.to_str().unwrap().to_string());

            // Extract Node and Flow-Package information
            let mut flow_package: FlowPackage =
                extract_flow_package_name_and_version(package_path).unwrap();
            debug(format!(
                "Flow-Package [name={}, version={}]",
                flow_package.name, flow_package.version
            ));
            flow_package.crates = extract_flow_crates(crate_package.clone(), package_path);
        }
    }
}
