use serde::{Deserialize, Serialize};
use serde_json;

use anyhow::{Error, Result};
use std::collections::HashMap;
use std::fs;

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
    pub constructors: HashMap<String, Constructor>,
}

impl Type {
    pub fn new_simple(name: &str, constructor: Constructor) -> Self {
        let mut t = Self {
            inputs: Option::None,
            outputs: Option::None,
            type_parameters: Option::None,
            constructors: HashMap::new()
        };
        t.constructors.insert(name.into(), constructor);
        t
    }
}

#[derive(Clone)]
pub struct Namespace {
    parts: Vec<String>,
}

impl Namespace {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    fn add_part(&mut self, part: &str) {
        self.parts.push(part.to_string());
    }

    fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
}

impl ToString for Namespace {
    fn to_string(&self) -> String {
        self.parts.join("_")
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum ArgumentPassing {
    Reference,
    MutableReference,
    Move,
    Clone,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Modifier {
    is_mutable: bool,
    is_reference: bool,
}

impl Modifier {
    fn nothing() -> Self {
        Self {
            is_mutable: false,
            is_reference: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ArgumentType {
    Type {
        name: String,
        type_parameters: Option<Vec<Box<ArgumentType>>>,
    },
    
    Generic {
        name: String,
    },
    //TODO: Tuple 
    // Tuple {
    //    type_parameters: Vec<Box<ArgumentType>>,
    //},
    //TODO: Enum 
}

impl ArgumentType {
    fn simple_type(name: &str) -> Box<ArgumentType> {
        Box::new(ArgumentType::Type {
            name: name.to_string(),
            type_parameters: None,
        })
    }

    fn simple_type_with_simple_typ_args(name: &str, tp_names: Vec<&str>) -> Box<ArgumentType> {
        Box::new(ArgumentType::Type {
            name: name.to_string(),
            type_parameters: Some(
                tp_names
                    .iter()
                    .map(|name| ArgumentType::simple_type(name))
                    .collect(),
            ),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ArgumentConstruction {
    Constructor(String),
    ExistingObject()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Argument {
    arg_type: Box<ArgumentType>,
    name: String,
    passing: ArgumentPassing,
    construction: ArgumentConstruction,
}

impl Argument {
    fn emit_prefix_code(&self) -> String {
        match self.passing {
            ArgumentPassing::Move => "".to_string(),
            ArgumentPassing::Clone => "".to_string(),
            ArgumentPassing::MutableReference => "&mut ".to_string(),
            ArgumentPassing::Reference => "&".to_string(),
        }
    }

    fn emit_postfix_code(&self) -> String {
        match self.passing {
            ArgumentPassing::Move => "".to_string(),
            ArgumentPassing::Clone => ".clone()".to_string(),
            ArgumentPassing::MutableReference => "".to_string(),
            ArgumentPassing::Reference => "".to_string(),
        }
    }

    fn new_change_observer_arg() -> Self {
        Self {
            arg_type: Box::new(ArgumentType::Type {
                name: "Option".to_string(),
                type_parameters: Some(vec![ArgumentType::simple_type("ChangeObserver")]),
                 //Hack: TODO: Add proper enum support.
            }),
            name: "change_observer".to_string(),
            passing: ArgumentPassing::Clone,
            construction: ArgumentConstruction::ExistingObject(),
        }
    }

    fn new_context_arg() -> Self {
        Self {
            arg_type: Box::new(ArgumentType::Type {
                name: "Arc".to_string(),
                type_parameters: Some(vec![ArgumentType::simple_type_with_simple_typ_args(
                    "Mutex",
                    vec!["flowrs::node::Context"],
                )]),
            }),
            name: "context".to_string(),
            passing: ArgumentPassing::Clone,
            construction: ArgumentConstruction::ExistingObject(),
        }
    }

    fn into_object(&self, type_name: &String, is_mutable: bool) -> ObjectDescription {
        ObjectDescription {
            type_name: type_name.clone(),
            type_parameters: HashMap::new(),
            name: self.name.clone(),
            is_mutable: is_mutable,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectDescription {
    pub type_name: String,
    pub type_parameters: HashMap<String, String>,
    pub name: String,
    pub is_mutable: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Constructor {
    New,
    NewWithObserver,
    NewWithObserverAndContext,
    NewWithArbitraryArgs(Vec<Argument>),
    FromJson,
    FromDefault,
}

impl Constructor {
    
    fn emit_type_parameters(&self, type_parameters: &Vec<String>, with_colons: bool) -> String {
        if type_parameters.is_empty() {
            return "".to_string();
        }

        format!(
            "<{}>{}",
            type_parameters
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<String>>()
                .join(", "),
            if with_colons { "::" } else { "" }
        )
    }

    fn get_fully_qualified_name(&self, name: &String, namespace: &Namespace, ignore: bool) -> String {
        if namespace.is_empty() || ignore {
            name.clone()
        } else {
            format!("{}_{}", namespace.to_string(), name)
        }
    }

    fn emit_args(&self, args: &Vec<Argument>, current_namespace: &Namespace) -> String {
        args.iter()
            .map(|arg| {
                format!(
                    "{}{}{}",
                    arg.emit_prefix_code(),
                    self.get_fully_qualified_name(&arg.name, current_namespace, matches!(arg.construction,ArgumentConstruction::ExistingObject())),
                    arg.emit_postfix_code()
                )
            })
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn get_resolved_arg_type_parameters(
        &self,
        arg_type: &Box<ArgumentType>,
        already_resolved_tps: &HashMap<String, String>,
        resolved_tps: &mut HashMap<String, String>,
    ) {
        match arg_type.as_ref() {
            ArgumentType::Type {
                type_parameters, ..
            } => {
                if let Some(type_params) = type_parameters {
                    for param in type_params {
                        self.get_resolved_arg_type_parameters(
                            param,
                            already_resolved_tps,
                            resolved_tps,
                        );
                    }
                }
            }
            
            ArgumentType::Generic { name } => {
                if let Some(resolved_name) = already_resolved_tps.get(name) {
                    /*
                    *arg_type = Box::new(ArgumentType::Type {
                        name: resolved_name.clone(),
                        modifier: modifier.clone(),
                        type_parameters: None,
                    });
                     */
                    resolved_tps.insert(name.clone(), resolved_name.clone());
                }
            }
        }
    }

    fn generate_arg_typename_rec(
        &self,
        type_name: &mut String,
        argument_type: &Box<ArgumentType>,
        resolved_type_parameters: &HashMap<String, String>,
    ) {
        match argument_type.as_ref() {
            ArgumentType::Type {
                name,
                type_parameters,
            } => {
                type_name.push_str(name);

                if let Some(type_params) = type_parameters {
                    type_name.push_str("<");

                    for tp in type_params {
                        self.generate_arg_typename_rec(type_name, tp, resolved_type_parameters);
                        type_name.push_str(",")
                    }

                    type_name.pop(); // pop last ,
                    type_name.push_str(">");
                }
            }
            ArgumentType::Generic { name } => {
                if let Some(tn) = resolved_type_parameters.get(name) {
                    type_name.push_str(tn);
                } else {
                    //TODO: Error Handling
                }
            }            
        }

        type_name.push_str(">");
    }

    fn generate_arg_typename(
        &self,
        type_name: &String,
        type_parameters: &Option<Vec<Box<ArgumentType>>>,
        resolved_type_parameters: &HashMap<String, String>,
    ) -> String {
        if let Some(type_params) = type_parameters {
            let mut res_type_name = type_name.clone();

            res_type_name.push_str("<");

            for tp in type_params {
                self.generate_arg_typename_rec(&mut res_type_name, tp, resolved_type_parameters);
                res_type_name.push_str(",")
            }

            res_type_name.pop(); // pop last ,
            res_type_name.push_str(">");

            res_type_name
        } else {
            type_name.clone()
        }
    }

    fn emit_arg_construction_code(
        &self,
        arg: &Argument,
        arg_constructor_name: String,
        pack_man: &PackageManager,
        current_namespace: &Namespace,
        resolved_type_parameters: &HashMap<String, String>,
    ) -> Result<String, Error> {
        match arg.arg_type.as_ref() {

            ArgumentType::Type {
                name,
                type_parameters,
            } => {
                if let Some(type_desc) = pack_man.get_type(name) {
                    
                    if let Some(arg_constructor) = type_desc.constructors.get(&arg_constructor_name) {

                        let object = arg.into_object(
                            &self.generate_arg_typename(name, type_parameters, resolved_type_parameters),
                            if let ArgumentPassing::MutableReference { .. } = arg.passing {
                                true
                            } else {
                                false
                            }, 
                        );

                        arg_constructor.emit_code_template(&object, pack_man, current_namespace)
                    } else {
                        Err(Error::msg(format!(
                            "Constructor '{}' for type '{}' not found.", arg_constructor_name, name 
                        ))) 
                    }
                } else {
                    Err(Error::msg(format!(
                        "Type description for '{}' not found.", name
                    )))
                }
            }

            ArgumentType::Generic { name } => {
                // get resolved type parameters.
                let mut resolved_tps = HashMap::<String, String>::new();
                self.get_resolved_arg_type_parameters(
                    &arg.arg_type,
                    resolved_type_parameters,
                    &mut resolved_tps,
                );

                // check if generic was already resolved. if so, try to get type and emit constructor code.
                // TODO: Think about what should happen if it is not yet resolved.
                if let Some(type_name) = resolved_tps.get(name) {
                    if let Some(type_desc) = pack_man.get_type(&type_name) {
                        
                        if let Some(arg_constructor) = type_desc.constructors.get(&arg_constructor_name) {

                            let object = arg.into_object(
                                &type_name,
                                if let ArgumentPassing::MutableReference { .. } = arg.passing {
                                    true
                                } else {
                                    false
                                },
                            );

                            arg_constructor.emit_code_template(
                                &object,
                                pack_man,
                                current_namespace,
                            )
                        } else {
                            Err(Error::msg(format!(
                                "Constructor '{}' for type '{}' not found.", arg_constructor_name, name 
                            ))) 
                        }
                    } else {
                        Err(Error::msg(format!(
                            "Type description for '{}' not found",
                            type_name
                        )))
                    }
                } else {
                    Err(Error::msg("Generic type was not resolved"))
                }
            }
        }
    }

    fn emit_args_construction_code(
        &self,
        pack_man: &PackageManager,
        args: &Vec<Argument>,
        current_namespace: &Namespace,
        resolved_type_parameters: &mut HashMap<String, String>,
    ) -> Result<String, Error> {
        let mut construction_blocks = Vec::<String>::new();
        
        for arg in args {
            // Only objects with a constructor need to be constructed.
            if let ArgumentConstruction::Constructor(constructor_name) = &arg.construction {

                // Generate construction for each argument.
                match self.emit_arg_construction_code(
                    arg,
                    constructor_name.clone(),
                    pack_man,
                    current_namespace,
                    resolved_type_parameters,
                ) {
                    Ok(code) => construction_blocks.push(code),
                    Err(err) => return Err(err),
                }
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
        od: &ObjectDescription,
        pack_man: &PackageManager,
        args: &Vec<Argument>,
        current_namespace: &Namespace,
    ) -> Result<String, Error> {
        if let Some(_) = pack_man.get_type(&od.type_name) {
            let mut new_namespace = current_namespace.clone();
            new_namespace.add_part(&od.name);

            let mut resolved_type_parameters = od.type_parameters.clone();

            let args_construction_code = self.emit_args_construction_code(
                pack_man,
                args,
                &new_namespace,
                &mut resolved_type_parameters,
            )?;

            Ok(format!(
                "{}\n let{} {} = {}::{}new({});",
                args_construction_code,
                self.emit_mutable(od.is_mutable),
                self.get_fully_qualified_name(&od.name, current_namespace, false),
                od.type_name,
                self.emit_type_parameters(&od.type_parameters.values().cloned().collect(), true),
                self.emit_args(args, &new_namespace)
            ))
        } else {
            Err(Error::msg(format!(
                "Type description for type '{}' not found",
                od.type_name
            )))
        }
    }

    fn emit_default(
        &self,
        od: &ObjectDescription,
        pack_man: &PackageManager,
        current_namespace: &Namespace,
    ) -> Result<String, Error> {
        if let Some(_) = pack_man.get_type(&od.type_name) {
            let full_object_name = self.get_fully_qualified_name(&od.name, current_namespace, false);

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

    fn emit_json_path(&self, cn: &Namespace, od: &ObjectDescription) -> String {
        format!(
            "{}[\"{}\"]",
            cn.parts
                .iter()
                .map(|item| format!("[\"{}\"]", item))
                .collect::<Vec<String>>()
                .join(""),
            od.name
        )
    }

    fn emit_new_from_json(
        &self,
        od: &ObjectDescription,
        pack_man: &PackageManager,
        current_namespace: &Namespace,
    ) -> Result<String, Error> {
        let full_object_name = self.get_fully_qualified_name(&od.name, current_namespace, false);

        Ok(format!(
            "let{} {}: {}{} = serde_json::from_value(data{}.clone());",
            self.emit_mutable(od.is_mutable),
            full_object_name,
            od.type_name,
            self.emit_type_parameters(&od.type_parameters.values().cloned().collect(), false),
            self.emit_json_path(current_namespace, od)
        ))
    }
}

impl Constructor  {
    pub fn emit_code_template(
        &self,
        obj_desc: &ObjectDescription,
        pack_man: &PackageManager,
        namespace: &Namespace,
    ) -> Result<String, Error> {
        match self {

            Self::New => self.emit_new_with_args(obj_desc, pack_man, &vec![], namespace),

            Self::NewWithObserver => self.emit_new_with_args(
                obj_desc,
                pack_man,
                &vec![Argument::new_change_observer_arg()],
                namespace,
            ),

            Self::NewWithObserverAndContext => self.emit_new_with_args(
                obj_desc,
                pack_man,
                &vec![
                    Argument::new_change_observer_arg(),
                    Argument::new_context_arg(),
                ],
                namespace,
            ),

            Self::NewWithArbitraryArgs(args) => {
                self.emit_new_with_args(obj_desc, pack_man, args, namespace)
            }

            Self::FromJson => self.emit_new_from_json(obj_desc, pack_man, namespace),

            Self::FromDefault => self.emit_default(obj_desc, pack_man, namespace),
            
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
                "constructors": 
                    {"New": "NewWithObserverAndContext"}
              }
            },
            "modules": {}
          }
        }
      }
    "#;

    let package_json_2 = r#"
    {
        "name":"my_package",
        "version":"1.0.0",
        "crates":{
           "my_crate":{
              "types":{
                 "ValueNode":{
                    "type_parameters":[
                       "T"
                    ],
                    "constructors":{
                       "New": 
                       {"NewWithArbitraryArgs":[
                          {
                             "arg_type":{
                                "Generic":{
                                   "name":"T"
                                }
                             },
                             "name":"value",
                             "passing":"MutableReference",
                             "construction":{"ExistingObject":[]}
                          }
                       ]
                    }
                    }
                 },

                 "ImageNode":{
                    "type_parameters":[
                       "T"
                    ],
                    "constructors":{
                        "New": {
                       "NewWithArbitraryArgs":[
                          {
                             "arg_type":{
                                "Type":{
                                   "name":"my_crate::ImageType",
                                   "type_parameters":[{
                                    "Generic":{
                                        "name":"T"
                                        }
                                    }
                                    ]
                                
                                }
                             },
                             "name":"image",
                             "passing":"MutableReference",
                             "construction":{"Constructor": "FromJson"}
                          }
                       ]
                    }
                    }
                 },
                 "ValueType":{
                    "constructors": {"FromJson": "FromJson"}
                 },


                 "ImageType":{
                    "constructors": {"FromJson": "FromJson"},
                    "type_parameters":[
                       "T"
                    ]
                 }
              },
              "modules":{
                 
              }
           }
        }
     }
    "#;

    let package_json_3 = r#"
    
    {
        "name":"flowrs-std",
        "version":"1.0.0",
        "crates":{
           "flowrs_std":{
              "types":{
                 
              },
              "modules":{
                 "nodes":{
                    "types":{
                       
                    },
                    "modules":{
                       "debug":{
                          "modules":{
                             
                          },
                          "types":{
                             "DebugNode":{
                                "inputs":[
                                   "input"
                                ],
                                "outputs":[
                                   "output"
                                ],
                                "type_parameters":[
                                   "I"
                                ],
                                "constructors":{
                                   "New":"NewWithObserver"
                                }
                             }
                          }
                       },
                       "value":{
                          "modules":{
                             
                          },
                          "types":{
                             "ValueType":{
                                "constructors":{
                                   "FromJson":"FromJson"
                                }
                             },
                             "ValueNode":{
                                "inputs":[
                                   
                                ],
                                "outputs":[
                                   "output"
                                ],
                                "type_parameters":[
                                   "I"
                                ],
                                "constructors":{
                                   "New":{
                                      "NewWithArbitraryArgs":[
                                         {
                                            "arg_type":{
                                               "Generic":{
                                                  "name":"I"
                                               }
                                            },
                                            "name":"value",
                                            "passing":"Move",
                                            "construction":{
                                               "Constructor":"Default"
                                            }
                                         },
                                         {
                                            "arg_type":{
                                               "Type":{
                                                  "name":"Option",
                                                  "type_parameters":[
                                                     {
                                                        "Type":{
                                                           "name":"ChangeObserver",
                                                           "type_parameters":null
                                                        }
                                                     }
                                                  ]
                                               }
                                            },
                                            "name":"Some(&change_observer)",
                                            "passing":"Move",
                                            "construction":{
                                               "ExistingObject":[
                                                  
                                               ]
                                            }
                                         }
                                      ]
                                   }
                                }
                             }
                          }
                       }
                    }
                 }
              }
           }
        }
     }
    
    
    "#;

    /*
    let arg1 = Argument {
        arg_type: ArgumentType::simple_type("my_crate::IntType"),
        name: "my_argument".to_string(),
        passing: ArgumentPassing::Move,
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
     */

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

    let package_1: Package = serde_json::from_str(&package_json).expect("wrong format.");
    let mut pm_1 = PackageManager::new();
    pm_1.add_package(package_1);
    let t_1 = pm_1.get_type("my_crate::MyType").expect("msg");
    let c_1 = t_1.constructors.get("New").expect("");
    let mut type_params_1 = HashMap::new();
    type_params_1.insert("U".to_string(), "i32".to_string());
    type_params_1.insert("T".to_string(), "i32".to_string());
    let mut ns_1 = Namespace::new();

    let obj_1 = ObjectDescription {
        type_name: "my_crate::MyType".to_string(),
        type_parameters: type_params_1.clone(),
        name: "value".to_string(),
        is_mutable: false,
    };
    println!("CODE: {}", c_1.emit_code_template(&obj_1, &pm_1, &ns_1).expect(""));    

    /*
    let a = Argument::new_change_observer_arg();
    let json = serde_json::to_string(&a).unwrap();
    println!("ARG: {}", json);

    let package: Package = serde_json::from_str(&package_json_3).expect("wrong format.");


    let mut type_params = HashMap::new();
    type_params.insert("T".to_string(), "i32".to_string());

    let mut pm = PackageManager::new();
    pm.add_package(package);

    let t = pm.get_type("my_crate::ImageNode");

    let obj = ObjectDescription {
        type_name: "my_crate::ImageNode".to_string(),
        type_parameters: type_params.clone(),
        name: "node1".to_string(),
        is_mutable: false,
    };

    let mut ns = Namespace::new();
    //ns.add_part("super");

    let arg = Argument::new_context_arg();
    println!("JSON: {}", serde_json::to_string(&arg).expect(""));

    println!("TEST");

    if let Some(ty) = t {
        println!(
            "Code: {}",
            ty.constructors.get("New".into()).expect("Constructor should be there.")
                .emit_code_template(&obj, &pm, &ns)
                .expect("Did not work!")
        );
    } */
     
}
