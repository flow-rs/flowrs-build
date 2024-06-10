use std::collections::{HashMap, LinkedList};
use std::io::{self, ErrorKind};
use std::num::NonZeroUsize;
use std::{fs, path::Path};
use syn::punctuated::Punctuated;
use syn::token::Impl;
use toml::{self, Value};

use cargo_metadata::{MetadataCommand, Package};

use syn::{
    AngleBracketedGenericArguments, FnArg, GenericArgument, GenericParam, Ident, Item, ItemImpl,
    ItemMod, ItemStruct, ItemType, PathArguments, PathSegment, PredicateType, Type, TypeParamBound,
    WhereClause,
};

use flowrs_package::flow_package::package::{Argument, Module as FlowModule};
use flowrs_package::flow_package::package::{ArgumentConstruction, Type as FlowType};
use flowrs_package::flow_package::package::{ArgumentPassing, Package as FlowPackage};
use flowrs_package::flow_package::package::{Constructor, Crate as FlowCrate};
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

fn input_to_parameter_list(input: &FnArg) -> Vec<(String, String, ArgumentPassing)> {
    let mut parameter_list: Vec<(String, String, ArgumentPassing)> = Vec::new();
    if let FnArg::Typed(pat_type) = input {
        if let syn::Pat::Ident(parameter_ident) = *pat_type.clone().pat {
            if let Type::Path(parameter_path) = *pat_type.ty.clone() {
                let syn::Path { segments, .. } = parameter_path.path.clone();
                let parameter_name = parameter_ident.ident.to_string();
                let parameter_type = segments[0].ident.to_string();
                debug(format!(
                    "ssssssssssssssssssssssssssssssssssssssssssssssssssssss{:?},{:?}, {:?}",
                    parameter_type, parameter_name, parameter_path
                ));
                let parameter_passing: ArgumentPassing;
                if parameter_ident.by_ref.is_some() && parameter_ident.mutability.is_some() {
                    parameter_passing = ArgumentPassing::MutableReference;
                } else if parameter_ident.by_ref.is_some() {
                    parameter_passing = ArgumentPassing::Reference;
                } else {
                    parameter_passing = ArgumentPassing::Clone;
                }
                // No way to identify move?
                parameter_list.push((parameter_type, parameter_name, parameter_passing));
            }
        }
    }
    return parameter_list;
}

fn convert_parameters_to_arguments(
    parameters: Vec<(String, String, ArgumentPassing)>,
    type_parameters: Option<Vec<TypeParameter>>,
) -> Vec<Argument> {
    return parameters
        .iter()
        .map(|(p_value, p_name, p_passing)| -> Argument {
            if p_name == "change_observer" {
                return Argument::new_change_observer_arg();
            } else if p_name == "context" {
                return Argument::new_context_arg();
            } else {
                let type_description: TypeDescription;
                let mut passing = p_passing.clone();
                let mut construction = ArgumentConstruction::ExistingObject();
                if type_parameters.is_some() {
                    let generics: Vec<String> = type_parameters
                        .clone()
                        .unwrap()
                        .iter()
                        .map(|tp| tp.name.clone())
                        .collect();

                    if generics.contains(p_value) {
                        type_description = TypeDescription::Generic {
                            name: p_value.to_string(),
                            type_parameters: None, // Currently no support for nested generics
                        };
                        passing = ArgumentPassing::Move;
                        construction = ArgumentConstruction::Constructor("Json".to_string());
                        debug(format!(
                            "GENERICS:{:?}, DESC:{:?},",
                            generics, type_description,
                        ));
                    } else {
                        type_description = TypeDescription::Type {
                            name: p_value.to_string(),
                            type_parameters: None, // Currently no support for nested generics
                        };
                    }
                } else {
                    type_description = TypeDescription::Type {
                        name: p_value.to_string(),
                        type_parameters: None, // Currently no support for nested generics
                    };
                }
                return Argument {
                    arg_type: Box::new(type_description),
                    name: p_name.to_string(),
                    passing: passing,
                    construction: construction,
                };
            }
        })
        .collect();
}

fn create_matching_constructor(
    fn_name: String,
    parameters: Vec<(String, String, ArgumentPassing)>,
    type_parameters: Option<Vec<TypeParameter>>,
) -> Constructor {
    if parameters.len() == 0 {
        // New Constructor
        return Constructor::New {
            function_name: Some(fn_name),
        };
    } else if parameters.len() == 1
        && parameters.contains(&(
            "Option".to_owned(),
            "change_observer".to_owned(),
            ArgumentPassing::Clone,
        ))
    {
        // NewWithObserver Constructor
        return Constructor::NewWithObserver {
            function_name: Some(fn_name),
        };
    } else if parameters.len() == 2
        && parameters.contains(&(
            "Option".to_owned(),
            "change_observer".to_owned(),
            ArgumentPassing::Clone,
        ))
        && parameters.contains(&(
            "Context".to_owned(),
            "context".to_owned(),
            ArgumentPassing::Clone,
        ))
    {
        // NewWithObserverAndContextConstructor
        return Constructor::NewWithObserverAndContext {
            function_name: Some(fn_name),
        };
    } else if parameters.len() == 1 {
        // FromJson Constructor
        return Constructor::FromJson;
    } else {
        //NewWithArbitraryArgs Constructor
        let arguments = convert_parameters_to_arguments(parameters, type_parameters);
        return Constructor::NewWithArbitraryArgs {
            function_name: Some(fn_name),
            arguments: arguments,
        };
    }
}

fn retrieve_constructors(
    function: &syn::ImplItemFn,
    type_parameters: Option<Vec<TypeParameter>>,
) -> (String, Constructor) {
    let parameters: Vec<(String, String, ArgumentPassing)> = function
        .sig
        .inputs
        .iter()
        .inspect(|i| {
            debug(format!(
                "SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS{:?}",
                i
            ))
        })
        .flat_map(|input: &FnArg| input_to_parameter_list(input))
        .collect();
    let fn_name = function.sig.ident.to_string().clone();

    parameters.iter().for_each(|(p1, p2, _)| {
        debug(format!(
            "ddddddddddddddddddddddddddddddddddddddddddddddddddddd{:?},{:?}",
            p1, p2
        ));
    });

    (
        fn_name.clone(),
        create_matching_constructor(fn_name, parameters, type_parameters),
    )
}

fn insert_constructors_to_type(
    implementation: &syn::ItemImpl,
    sub_types: &mut HashMap<String, FlowType>,
    function: &syn::ImplItemFn,
) {
    let implemented_type = *implementation.self_ty.clone();
    if let syn::Type::Path(impl_ty_path) = implemented_type {
        if let Some(segment) = impl_ty_path.path.segments.first() {
            if let Some(sub_type) = sub_types.get_mut(&segment.ident.to_string()) {
                // debug(format!(
                //     "KKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKKK{:?}",
                //     sub_type
                // ));
                // At this Point we have retrieved the sub-type
                let (constructor_type, constructor) =
                    retrieve_constructors(function, sub_type.type_parameters.clone());
                sub_type.constructors.insert(constructor_type, constructor);
            }
        }
    }
}

// Extracts all constructor functions by looking at for the return type "Self"
fn extract_constructor_functions(
    sub_types: &mut HashMap<String, FlowType>,
    sub_impls: &LinkedList<ItemImpl>,
) {
    for implementation in sub_impls {
        for item in &implementation.items {
            match item {
                syn::ImplItem::Fn(function) => {
                    if let syn::ReturnType::Type(_, type_box) = &function.sig.output {
                        if let syn::Type::Path(type_path) = *type_box.clone() {
                            if type_path.path.is_ident("Self") {
                                // At this Point we have a constructor function
                                insert_constructors_to_type(implementation, sub_types, function)
                            }
                        }
                    }
                }
                // }
                _ => {} // Other items are not relevant
            }
        }
    }
}

fn parse_module(module: ItemMod, path: &Path) -> FlowModuleWrapper {
    // Define necessary output variables
    let module_name = module.ident.to_string();
    let mut sub_modules: HashMap<String, FlowModule> = HashMap::new();
    let mut sub_types: HashMap<String, FlowType> = HashMap::new();
    let mut sub_impls: LinkedList<ItemImpl> = LinkedList::new();

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
                //debug(format!("PARSING SUBMODULE [{:?}]", m.clone()));
                let sub_module_wrapper = parse_module(m, &path.join(module_name.clone()));
                sub_modules.insert(sub_module_wrapper.name, sub_module_wrapper.flow_module);
            }
            Item::Struct(t) => {
                //debug(format!("PARSING SUBTYPE[{:?}]", t.clone()));
                let sub_type_wrapper = parse_type(t);
                sub_types.insert(sub_type_wrapper.name, sub_type_wrapper.flow_type);
            }
            Item::Impl(i) => {
                //debug(format!("PARSING IMPLEMENTATION[{:?}]", i.clone()));
                sub_impls.push_back(i);
            }
            _ => {} // Other items are not relevant
        }
    }

    // Extract constructors from implementations
    extract_constructor_functions(&mut sub_types, &sub_impls);

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
    let type_parameters: Vec<TypeParameter>;
    let constructors = HashMap::new();

    // Parse type syntax structure
    // debug(format!(
    //     "TYPE: [type_name: {}, type_structure: {:?}",
    //     type_name, itemtype
    // ));

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

    // Extract inputs and outputs
    let fields = itemtype.fields;
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
                    // debug(format!(
                    //     "Input field: {} with type: {}",
                    //     field_name, generic_type
                    // ));
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
                    // debug(format!(
                    //     "Output field: {} with type: {}",
                    //     field_name, generic_type
                    // ));
                }
            }
        }
    }

    // Extract constructors

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

// fn extract_constructors(crate_without_constructors: (&String, &FlowCrate)) -> (String, FlowCrate) {
//     let mut crate_with_constructors: FlowCrate = crate_without_constructors.1.to_owned();
//     //crate_with_constructors.
//     (
//         crate_without_constructors.0.to_string(),
//         crate_with_constructors,
//     )
// }

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
            // // Use second pass to extract constructors
            // flow_package
            //     .crates
            //     .iter()
            //     .map(|crate_| extract_constructors(crate_));
            let package_json = serde_json::to_string(&flow_package);
            debug(package_json.unwrap());
        }
    }
}
