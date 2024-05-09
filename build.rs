use std::{error::Error, fs, path::Path};

use cargo_metadata::{MetadataCommand, Package};

use syn::{File, Item};

// Function to determine if a Cargo Package is a flow-package
fn is_flow_package(package: Package) -> bool {
    package.keywords.contains(&"flow-package".to_string())
}

// Function to read flow-nodes from Cargo-Package
fn extract_flow_nodes(package: Package, package_path: &Path) -> Result<(), Box<dyn Error>> {
    let src_path = package_path.join("src");
    //let nodes_path = src_path.join("nodes");
    let lib_path = src_path.join("lib.rs");

    // Parse lib.rs to get the module structure
    let file_content = fs::read_to_string(lib_path).expect("Unable to read lib.rs");
    let syntax_tree = syn::parse_file(&file_content)?;
    for item in syntax_tree.items {
        match item {
            Item::Mod(m) => todo!(),
            Item::Const(_) => todo!(),
            Item::Enum(_) => todo!(),
            Item::ExternCrate(_) => todo!(),
            Item::Fn(_) => todo!(),
            Item::ForeignMod(_) => todo!(),
            Item::Impl(_) => todo!(),
            Item::Macro(_) => todo!(),
            Item::Static(_) => todo!(),
            Item::Struct(_) => todo!(),
            Item::Trait(_) => todo!(),
            Item::TraitAlias(_) => todo!(),
            Item::Type(_) => todo!(),
            Item::Union(_) => todo!(),
            Item::Use(_) => todo!(),
            Item::Verbatim(_) => todo!(),
            _ => todo!(),
        }
    }

    Ok(())
}

fn main() {
    // Fetch Metadata from the cargo.toml
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to fetch cargo metadata");

    for package in metadata.packages {
        // Filter for dependencies containing nodes
        if is_flow_package(package.clone()) {
            let package_path = Path::new(&package.manifest_path).parent().unwrap();
            println!("cargo::warning={}", package_path.to_str().unwrap());
            extract_flow_nodes(package.clone(), package_path);
        }
    }
}
