use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::{fs, path::Path};
use toml::{self, Value};

use cargo_metadata::{MetadataCommand, Package};

use syn::{
    GenericArgument, GenericParam, Ident, Item, ItemMod, ItemStruct, ItemType, PathArguments,
    PredicateType, Type, TypeParamBound, WhereClause,
};

use flowrs_package::flow_package::package::Crate as FlowCrate;
use flowrs_package::flow_package::package::Module as FlowModule;
use flowrs_package::flow_package::package::Package as FlowPackage;
use flowrs_package::flow_package::package::Type as FlowType;
use flowrs_package::flow_package::package::{Input, Output, TypeDescription, TypeParameter};
//use flowrs_package::flow_package::package_manager::PackageManager as FlowPackageManager;

const DEBUG_STR: &str = "cargo::warning= [DEBUG]:";

fn debug(message: String) {
    let debug = true;

    if debug {
        println!("{} {}", DEBUG_STR, message)
    }
}

#[derive(Debug)]
pub struct FlowModuleWrapper {
    pub name: String,
    pub flow_module: FlowModule,
}

#[derive(Debug)]
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
fn extract_flow_crates(cargo_package: Package, package_path: &Path) -> HashMap<String, FlowCrate> {
    // Read file
    let src_path = package_path.join("src");
    let lib_path = src_path.join("lib.rs");

    let file_content = fs::read_to_string(lib_path).expect("Unable to read lib.rs");
    let lib_tree = syn::parse_file(&file_content).expect("Unable to parse lib.rs file content");

    // Define necessary output variables
    let mut crates: HashMap<String, FlowCrate> = HashMap::new();
    let mut sub_types: HashMap<String, FlowType> = HashMap::new();
    let mut sub_modules: HashMap<String, FlowModule> = HashMap::new();

    // Parse lib.rs to get the module structure
    for item in lib_tree.items {
        match item {
            Item::Mod(m) => {
                debug(format!("PARSING MODULE [{:?}]", m.clone()));
                let module_wrapper = parse_module(m, &src_path);
                sub_modules.insert(module_wrapper.name, module_wrapper.flow_module);
            }
            Item::Struct(t) => {
                debug(format!("PARSING TYPE[{:?}]", t.clone()));
                let type_wrapper = parse_type(t);
                sub_types.insert(type_wrapper.name, type_wrapper.flow_type);
            }
            _ => (), // Other Items are not relevant
        }
    }

    // Return result
    crates.insert(
        cargo_package.name,
        FlowCrate {
            types: sub_types,
            modules: sub_modules,
        },
    );

    crates
}

fn parse_module(module: ItemMod, path: &Path) -> FlowModuleWrapper {
    // Define necessary output variables
    let module_name = module.ident.to_string();
    let mut sub_modules: HashMap<String, FlowModule> = HashMap::new();
    let mut sub_types: HashMap<String, FlowType> = HashMap::new();

    // Read file
    let module_file_path = path.join(module_name.clone()).with_extension("rs");
    let file_content = fs::read_to_string(module_file_path.clone()).expect(
        format!(
            "Unable to read module file at {:?}",
            module_file_path.to_str()
        )
        .as_str(),
    );
    let module_tree = syn::parse_file(&file_content)
        .expect(format!("Unable to parse {}.rs file content", module_name.clone()).as_str());

    // Parse module syntax structure
    for item in module_tree.items {
        match item {
            Item::Mod(m) => {
                debug(format!("PARSING SUBMODULE [{:?}]", m.clone()));
                let sub_module_wrapper = parse_module(m, &path.join(module_name.clone()));
                sub_modules.insert(sub_module_wrapper.name, sub_module_wrapper.flow_module);
            }
            Item::Struct(t) => {
                debug(format!("PARSING SUBTYPE[{:?}]", t.clone()));
                let sub_type_wrapper = parse_type(t);
                sub_types.insert(sub_type_wrapper.name, sub_type_wrapper.flow_type);
            }
            _ => (), // Other Items are not relevant
        }
    }

    // Return result
    let flow_module = FlowModule {
        types: sub_types,
        modules: sub_modules,
    };

    FlowModuleWrapper {
        name: module_name,
        flow_module: flow_module,
    }
}

// See https://stackoverflow.com/a/56264023
fn extract_type_path(ty: &syn::Type) -> Option<&syn::Path> {
    match *ty {
        syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
        _ => None,
    }
}

fn extract_generic(path: &syn::Path) -> Option<syn::Ident> {
    // Generic segment should be the last segment of the syn::Path
    if let Some(last_segment) = path.segments.last() {
        if let PathArguments::AngleBracketed(ref pathArg) = last_segment.arguments {
            for arg in pathArg.args.iter() {
                if let GenericArgument::Type(Type::Path(ref generic_type)) = arg {
                    debug(format!(
                        "OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO{:?}",
                        generic_type
                    ));
                    //return Some(last_segment.ident.clone());
                    return Some(generic_type.path.segments.last().unwrap().ident.clone());
                }
            }
        }
    }
    None
}

fn extract_matching_constraints(
    all_constraints: Option<WhereClause>,
    type_ident: Ident,
) -> Vec<String> {
    let mut constraints: Vec<String> = Vec::new();
    if let Some(where_clause) = all_constraints {
        constraints = where_clause
            .predicates
            .iter()
            // We do not allow lifetime parameters or other where clause types
            .filter_map(|predicate| {
                if let syn::WherePredicate::Type(pred_ty) = predicate {
                    if let Type::Path(path) = &pred_ty.bounded_ty {
                        if let Some(ident) = path
                            .path
                            .segments
                            .last()
                            .map(|segment| segment.ident.clone())
                        {
                            if type_ident.eq(&ident) {
                                Some(pred_ty.bounds.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .map(|bounds| {
                bounds
                    .iter()
                    .filter_map(|type_param_bound| {
                        if let TypeParamBound::Trait(trait_bound) = type_param_bound {
                            trait_bound.path.get_ident()
                        } else {
                            None
                        }
                    })
                    .map(|ident| ident.to_string())
                    .collect()
            })
            .collect();
    }
    constraints
}

fn parse_type(itemtype: ItemStruct) -> FlowTypeWrapper {
    // Define necessary output variables
    let type_name = itemtype.ident.to_string();
    let mut inputs: HashMap<String, Input> = HashMap::new();
    let mut outputs: HashMap<String, Output> = HashMap::new();
    let mut type_parameters: Vec<TypeParameter> = Vec::new();
    let constructors = HashMap::new();

    // Parse type syntax structure
    debug(format!(
        "TYPE: [type_name: {}, type_structure: {:?}",
        type_name, itemtype
    ));

    // Extract generic parameter and where clause constraints
    let all_constraints = itemtype.generics.where_clause;
    type_parameters = itemtype
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(ty) => Some(TypeParameter {
                name: ty.ident.to_string(),
                constraints: extract_matching_constraints(
                    all_constraints.clone(),
                    ty.ident.clone(),
                ),
            }),
            _ => None,
        })
        .collect();

    let fields = itemtype.fields;
    //let mut input_fields: HashMap<String, Input> = HashMap::new();
    for field in fields {
        let field_attrs = field.attrs;
        //let field_mutability = field.mutability;
        let field_type = field.ty;
        let field_name = field.ident.unwrap().to_string();

        // debug(format!(
        //     "OLAKSJDLKAJSLKDKLÖASD{:?}",
        //     extract_type_path(&field_type).unwrap().to_owned()
        // ));

        for attr in field_attrs {
            // Check for #[input] and #[output] other fields are not relevant
            if attr.path().is_ident("input") {
                // Field is correctly identified as input field
                if let Some(generic_type) =
                    extract_type_path(&field_type).and_then(|path| extract_generic(path))
                {
                    inputs.insert(
                        field_name.clone(),
                        Input {
                            input_type: TypeDescription::Generic {
                                name: generic_type.to_string(),
                                type_parameters: None,
                            },
                        },
                    );
                    debug(format!(
                        "Input field: {} with type: {}",
                        field_name, generic_type
                    ));
                }
            } else if attr.path().is_ident("output") {
                // Field is correctly identified as output field
                if let Some(generic_type) =
                    extract_type_path(&field_type).and_then(|path| extract_generic(path))
                {
                    outputs.insert(
                        field_name.clone(),
                        Output {
                            output_type: TypeDescription::Generic {
                                name: generic_type.to_string(),
                                type_parameters: None,
                            },
                        },
                    );
                    debug(format!(
                        "Output field: {} with type: {}",
                        field_name, generic_type
                    ));
                }
            }
        }
    }

    // Return result
    let flow_type = FlowType {
        inputs: Some(inputs),
        outputs: Some(outputs),
        type_parameters: Some(type_parameters),
        constructors: constructors,
    };
    FlowTypeWrapper {
        name: type_name,
        flow_type: flow_type,
    }
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
            // Extract Node and Flow-Package information
            let mut flow_package: FlowPackage =
                extract_flow_package_name_and_version(package_path).unwrap();
            debug(format!(
                "PARSING FLOW-PACKAGE [name={}, version={}, path={}]",
                flow_package.name,
                flow_package.version,
                package_path.to_str().unwrap().to_string()
            ));
            flow_package.crates = extract_flow_crates(crate_package.clone(), package_path);
            let package_json = serde_json::to_string(&flow_package);
            debug(package_json.unwrap());
        }
    }
}
