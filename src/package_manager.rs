use std::collections::HashMap;
use std::fs;

use crate::package::{Crate, Constructor, Package, Type};

pub struct PackageManager {
    packages: HashMap<String, Package>,
}

impl PackageManager {
    pub fn new() -> Self {
        let mut pm = Self {
            packages: HashMap::new(),
        };

        pm.add_built_in_package();

        pm
    }

    pub fn new_from_folder(directory_path: &str) -> Self {
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
        let mut pm = PackageManager { packages: packages };
        pm.add_built_in_package();

        pm
    }

    fn add_built_in_package(&mut self) {
        let prims: [&str; 16] = [
            "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "isize", "usize",
            "f32", "f64", "bool", "char",
        ];

        let mut types = HashMap::new();
        for prim in prims {
            types.insert(
                prim.to_string(),
                Type::new_simple("Default".into(), Constructor::FromDefault,),
            );
        }

        let mut crates = HashMap::new();
        crates.insert("primitives".to_string(), Crate::new_with_types(types));

        self.add_package(Package {
            name: "built-in".to_string(),
            version: "1.0.0".to_string(),
            crates: crates,
        })
    }

    pub fn add_package(&mut self, package: Package) {
        if !self.packages.contains_key(&package.name) {
            self.packages.insert(package.name.clone(), package);
        }
    }

    pub fn get_all_packages(&self) -> Vec<Package> {
        self.packages.values().cloned().collect()
    }

    pub fn get_package(&self, package_name: &str) -> Option<&Package> {
        self.packages.get(package_name)
    }

    pub fn get_type(&self, type_name: &str) -> Option<&Type> {
        let type_ids: Vec<&str> = type_name.split("::").collect();

        // check built-in types.
        if type_ids.len() == 1 {
            return self
                .packages
                .get("built-in")
                .expect("built-in package not available.")
                .crates
                .get("primitives")
                .expect("primitives crate not available.")
                .types
                .get(type_ids[0]);
        }

        // iterate over packages and return type if available.
        // Note: We cannot handle same crate, same type, different package situations.
        for (_, p) in &self.packages {
            let res = self.get_type_from_package(&type_ids, p);
            if res.is_some() {
                return res;
            }
        }

        Option::None
    }

    pub fn get_type_from_package<'a>(
        &self,
        type_ids: &Vec<&str>,
        package: &'a Package,
    ) -> Option<&'a Type> {
        // We need at least 2 parts of the name crate::type.
        if type_ids.len() < 2 {
            return Option::None;
        }

        if let Some(cr) = package.crates.get(type_ids[0]) {
            // Crates can have types.
            if type_ids.len() == 2 {
                return cr.types.get(type_ids[1]);
            }

            if let Some(mut module) = cr.modules.get(type_ids[1]) {
                // get module.

                // first 2 name parts are crate::module so skip them
                for (index, type_id) in type_ids.iter().enumerate().skip(2) {
                    if index == type_ids.len() - 1 {
                        //last iteration.
                        return module.types.get(*type_id);
                    } else {
                        // set current module to the module's child module.
                        if let Some(m) = module.modules.get(*type_id) {
                            module = m;
                        } else {
                            return Option::None;
                        }
                    }
                }
            }
        }
        Option::None
    }
}
