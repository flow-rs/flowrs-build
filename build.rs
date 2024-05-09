use std::path::Path;

use cargo_metadata::{MetadataCommand, Package};

fn is_flow_package_repository(package: Package) -> bool {
    package.keywords.contains(&"flow-package".to_string())
}

fn main() {
    // Fetch Metadata from the cargo.toml
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to fetch cargo metadata");

    for package in metadata.packages {
        // Filter for dependencies containing nodes
        if is_flow_package_repository(package.clone()) {
            let package_path = Path::new(&package.manifest_path).parent().unwrap();
            println!("cargo::warning={}", package_path.to_str().unwrap());
        }
    }
}
