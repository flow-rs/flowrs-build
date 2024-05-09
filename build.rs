use std::collections::HashMap;
use std::{error::Error, fs, path::Path};

use cargo_metadata::{MetadataCommand, Package};

use syn::{File, Item, ItemMod};

use flowrs_package::flow_package::package::Crate as FlowCrate;
use flowrs_package::flow_package::package::Package as FlowPackage;
use flowrs_package::flow_package::package_manager::PackageManager as FlowPackageManager;

// Function to determine if a Cargo Package is a flow-package
fn is_flow_package(package: Package) -> bool {
    package.keywords.contains(&"flow-package".to_string())
}

// Crates new FlowPackage and adds extracted name and version from crate package
fn extract_flow_package_name_and_version(package_path: &Path) -> FlowPackage {
    FlowPackage::default()
}

// Function to extract flow-crates from cargo-package
fn extract_flow_crates(cargo_package: Package, package_path: &Path) -> HashMap<String, FlowCrate> {
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

fn parse_module(module: ItemMod) {}

fn main() {
    // Fetch Metadata from the cargo.toml
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to fetch cargo metadata");

    for crate_package in metadata.packages {
        // Filter for dependencies containing nodes
        if is_flow_package(crate_package.clone()) {
            let package_path = Path::new(&crate_package.manifest_path).parent().unwrap();
            println!("cargo::warning={}", package_path.to_str().unwrap());

            // Extract Node and Flow-Package information
            let mut flow_package: FlowPackage = extract_flow_package_name_and_version(package_path);
            flow_package.crates = extract_flow_crates(crate_package.clone(), package_path);
        }
    }
}
