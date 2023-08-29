use serde::{Deserialize, Serialize};
use serde_json;

use anyhow::{Error, Result};
use std::fs;
use std::{collections::HashMap};

use crate::package_manager::PackageManager;

#[derive(Serialize, Deserialize, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub crates: HashMap<String, Crate>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Crate {
    pub types: HashMap<String, Type>,
    pub modules: HashMap<String, Module>,
    //Note: We do not allow sub-crates.
    //      All we care about are correct full qualified type names.
    //      And in Rust, parent crates are not part of the fqn of a type.
}

impl Crate {
    pub fn new_with_types(types: HashMap<String, Type>) -> Self {
        Self {
            types: types,
            modules: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Module {
    pub types: HashMap<String, Type>,
    pub modules: HashMap<String, Module>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Type {
    pub inputs: Option<Vec<String>>,
    pub outputs: Option<Vec<String>>,
    pub type_parameters: Option<Vec<String>>,
    pub constructor: DynamicConstructor,
}

impl Type {
    pub fn new_simple(constructor: DynamicConstructor) -> Self {
        Self {
            inputs: Option::None,
            outputs: Option::None,
            type_parameters: Option::None,
            constructor: constructor,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum ArgumentPassing {
    Reference,
    MutableReference,
    Value,
    Clone,
}

impl ArgumentPassing {
    fn emit_prefix_code(&self) -> String {
        match self {
            ArgumentPassing::Reference => "&".to_string(),
            ArgumentPassing::MutableReference => "&mut ".to_string(),
            ArgumentPassing::Value => "".to_string(),
            ArgumentPassing::Clone => "".to_string(),
        }
    }

    fn emit_postfix_code(&self) -> String {
        match self {
            ArgumentPassing::Reference => "".to_string(),
            ArgumentPassing::MutableReference => "".to_string(),
            ArgumentPassing::Value => "".to_string(),
            ArgumentPassing::Clone => ".clone()".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Argument {
    type_name: String,
    type_parameters: Vec<String>,

    name: String,

    passing: ArgumentPassing,
    existing_object: bool,
}

impl Argument {
    fn new_change_observer_arg() -> Self {
        Self {
            type_name: "Option<flowrs::node::ChangeObserver>".to_string(),
            type_parameters: vec![],
            name: "Some(&change_observer)".to_string(), //Hack: TODO: Add proper Option support.
            passing: ArgumentPassing::Value,
            existing_object: true,
        }
    }

    fn new_context_arg() -> Self {
        Self {
            type_name: "Arc<Mutex<flowrs::node::Context>>".to_string(),
            type_parameters: vec![],
            name: "context".to_string(),
            passing: ArgumentPassing::Clone,
            existing_object: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Object {
    pub type_name: String,
    pub type_parameters: Vec<String>,
    pub name: String,
    pub is_mutable: bool,
}

impl From<&Argument> for Object {
    fn from(argument: &Argument) -> Self {
        Object {
            type_name: argument.type_name.clone(),
            type_parameters: argument.type_parameters.clone(),
            name: argument.name.clone(),
            is_mutable: match argument.passing {
                ArgumentPassing::MutableReference => true,
                _ => false,
            },
        }
    }
}

pub trait Constructor {
    fn emit_code_template(
        &self,
        object_desc: &Object,
        pack_man: &PackageManager,
        namespace: &String,
    ) -> Result<String, Error>;
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DynamicConstructor {
    New,
    NewWithObserver,
    NewWithObserverAndContext,
    NewWithArbitraryArgs(Vec<Argument>),
    FromJson,
    FromDefault,
}

impl DynamicConstructor {
    fn emit_type_parameters(&self, type_parameters: &Vec<String>) -> String {
        if type_parameters.is_empty() {
            return "".to_string();
        }

        format!(
            "<{}>::",
            type_parameters
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    fn get_full_name(&self, name: &String, namespace: &String, ignore: bool) -> String {
        if namespace.is_empty() || ignore {
            name.clone()
        } else {
            format!("{}_{}", namespace, name)
        }
    }

    fn emit_args(&self, args: &Vec<Argument>, current_namespace: &String) -> String {
        args.iter()
            .map(|arg| {
                format!(
                    "{}{}{}",
                    arg.passing.emit_prefix_code(),
                    self.get_full_name(&arg.name, current_namespace, arg.existing_object),
                    arg.passing.emit_postfix_code()
                )
            })
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn emit_arg_construction_code(
        &self,
        arg: &Argument,
        pack_man: &PackageManager,
        current_namespace: &String,
    ) -> Result<String, Error> {
        if let Some(type_desc) = pack_man.get_type(&arg.type_name) {
            type_desc
                .constructor
                .emit_code_template(&arg.into(), pack_man, current_namespace)
        } else {
            Err(Error::msg("Type description not found"))
        }
    }

    fn emit_args_construction_code(
        &self,
        pack_man: &PackageManager,
        args: &Vec<Argument>,
        current_namespace: &String,
    ) -> Result<String, Error> {
        let mut construction_blocks = Vec::<String>::new();
        for arg in args {
            if arg.existing_object {
                continue;
            }

            match self.emit_arg_construction_code(arg, pack_man, current_namespace) {
                Ok(code) => construction_blocks.push(code),
                Err(err) => return Err(err),
            }
        }

        Ok(construction_blocks.join("\n"))
    }

    fn emit_mutable(&self, is_mutable: bool) -> String {
        if is_mutable {
            " mut".to_string()
        } else {
            "".to_string()
        }
    }

    fn emit_new_with_args(
        &self,
        od: &Object,
        pack_man: &PackageManager,
        args: &Vec<Argument>,
        current_namespace: &String,
    ) -> Result<String, Error> {
        if let Some(_) = pack_man.get_type(&od.type_name) {
            let full_object_name = self.get_full_name(&od.name, current_namespace, false);
            let arg_construction_code =
                self.emit_args_construction_code(pack_man, args, &full_object_name)?;

            Ok(format!(
                "{}\n let{} {} = {}::{}new({});",
                arg_construction_code,
                self.emit_mutable(od.is_mutable),
                full_object_name,
                od.type_name,
                self.emit_type_parameters(&od.type_parameters),
                self.emit_args(args, &full_object_name)
            ))
        } else {
            Err(Error::msg("Type description not found"))
        }
    }

    fn emit_default(
        &self,
        od: &Object,
        pack_man: &PackageManager,
        current_namespace: &String,
    ) -> Result<String, Error> {
        if let Some(_) = pack_man.get_type(&od.type_name) {
            let full_object_name = self.get_full_name(&od.name, current_namespace, false);

            Ok(format!(
                "let{} {}:{} = Default::default();",
                self.emit_mutable(od.is_mutable),
                full_object_name,
                od.type_name
            ))
        } else {
            Err(Error::msg("Type description not found"))
        }
    }

    //TODO
    fn emit_new_from_json(
        &self,
        object_desc: &Object,
        pack_man: &PackageManager,
    ) -> Result<String, Error> {
        //format!("{{object_name}}:{}<{}>=serde_json::from_str({{json_string}}).unwrap();", type_name, self.get_type_parameters_str(type_desc))
        Ok("".to_string()) //TODO
    }
}

impl Constructor for DynamicConstructor {
    fn emit_code_template(
        &self,
        object_desc: &Object,
        pack_man: &PackageManager,
        namespace: &String,
    ) -> Result<String, Error> {
        match self {
            Self::New => self.emit_new_with_args(object_desc, pack_man, &vec![], namespace),
            Self::NewWithObserver => self.emit_new_with_args(
                object_desc,
                pack_man,
                &vec![Argument::new_change_observer_arg()],
                namespace,
            ),
            Self::NewWithObserverAndContext => self.emit_new_with_args(
                object_desc,
                pack_man,
                &vec![
                    Argument::new_change_observer_arg(),
                    Argument::new_context_arg(),
                ],
                namespace,
            ),
            Self::NewWithArbitraryArgs(args) => {
                self.emit_new_with_args(object_desc, pack_man, args, namespace)
            }
            Self::FromJson => self.emit_new_from_json(object_desc, pack_man),
            Self::FromDefault => self.emit_default(object_desc, pack_man, namespace),
        }
    }
}

#[test]
fn test() {
    let package_json = r#"
    {
        "name": "my_package",
        "version": "1.0.0",
        "crates": {
          "my_crate": {
            "types": {
              "MyType": {
                "inputs": null,
                "outputs": null,
                "type_parameters": ["U", "T"],
                "constructor": 
                    "NewWithObserverAndContext"
              }
            },
            "modules": {}
          }
        }
      }
    "#;

    let package_json_2 = r#"
    {
        "name": "my_package",
        "version": "1.0.0",
        "crates": {
          "my_crate": {
            "types": {
              "MyType": {
                "constructor": {
                  "NewWithArbitraryArgs": [
                    {
                      "type_name": "my_package::my_crate::IntType",
                      "type_parameters": [],
                      "name": "my_argument",
                      "passing": "MutableReference",
                      "existing_object": false
                    }
                  ]
                }
              },
              "IntType": {
                "constructor": "FromDefault"
              }
            },
            "modules": {}
          }
        }
      }
      
    "#;

    let arg1 = Argument {
        type_name: "my_package::my_crate::IntType".to_string(),
        type_parameters: vec!["T".to_string(), "U".to_string()],
        name: "my_argument".to_string(),
        passing: ArgumentPassing::Reference,
        existing_object: false,
    };

    let args = vec![arg1];

    let my_type = Type {
        inputs: Option::None,
        outputs: Option::None,
        type_parameters: Option::None,
        constructor: DynamicConstructor::NewWithArbitraryArgs(args),
    };

    let int_type = Type {
        inputs: Option::None,
        outputs: Option::None,
        type_parameters: Option::None,
        constructor: DynamicConstructor::New,
    };

    /*
    let mut types = HashMap::<String, Type>::new();
    types.insert("MyType".into(), my_type);
    types.insert("IntType".into(), int_type);

    let mut modules = HashMap::<String, Module>::new();

    let my_crate = Crate{ types: types, modules: modules};

    let mut crates = HashMap::<String, Crate>::new();
    crates.insert("my_crate".into(), my_crate);

    let p = Package { name: "my_package".into(), version: "1.0.0".into(), crates: crates};

    let json = serde_json::to_string(&p).unwrap();
    println!("{}", json);
     */

    let package: Package = serde_json::from_str(&package_json_2).expect("wrong format.");

    let mut pm = PackageManager::new();
    pm.add_package(package);

    let t = pm.get_type("my_package::my_crate::MyType");

    let obj = Object {
        type_name: "my_package::my_crate::MyType".to_string(),
        type_parameters: vec!["i32".into(), "i32".into()],
        name: "my_object".to_string(),
        is_mutable: false,
    };

    if let Some(ty) = t {
        println!(
            "Code: {}",
            ty.constructor
                .emit_code_template(&obj, &pm, &"super_namespace".to_string())
                .expect("Did not work!")
        );
    }
}
