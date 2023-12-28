use serde::{Deserialize, Serialize};

use anyhow::{Error, Result};
use std::collections::HashMap;
use handlebars::Handlebars;

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
pub struct Input {
    #[serde(rename = "type")]
    input_type: TypeDescription     
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Output {
    #[serde(rename = "type")]
    output_type: TypeDescription     
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Type {
    pub inputs: Option<HashMap<String, Input>>,
    pub outputs: Option<HashMap<String, Output>>,
    pub type_parameters: Option<Vec<String>>,
    pub constructors: HashMap<String, Constructor>,
}

impl Type {
    pub fn new_with_constructor(constructor_name: &str, constructor: Constructor) -> Self {
        let mut t = Self {
            inputs: Option::None,
            outputs: Option::None,
            type_parameters: Option::None,
            constructors: HashMap::new()
        };
        t.constructors.insert(constructor_name.into(), constructor);
        t
    }

    pub fn new_prelude_type() -> Self {
        let mut t = Self {
            inputs: Option::None,
            outputs: Option::None,
            type_parameters: Option::None,
            constructors: HashMap::new()
        };
        t.constructors.insert("Default".into(), Constructor::FromDefault);
        t.constructors.insert("Json".into(), Constructor::FromJson);

        t
    }

    pub fn new_simple() -> Self {
        Self {
            inputs: Option::None,
            outputs: Option::None,
            type_parameters: Option::None,
            constructors: HashMap::new()
        }
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
pub enum TypeDescription {
    Type {
        name: String,
        type_parameters: Option<Vec<Box<TypeDescription>>>,
    },
    
    Generic {
        name: String,
        type_parameters: Option<Vec<Box<TypeDescription>>>,
    },

    //TODO: Tuple 
    // Tuple {
    //    type_parameters: Vec<Box<ArgumentType>>,
    //},

    //TODO: Enum 
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ArgumentConstruction {
    Constructor(String),
    ExistingObject()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Argument {
    #[serde(rename = "type")]
    arg_type: Box<TypeDescription>,
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
            arg_type: Box::new(TypeDescription::Type {
                name: "()".to_string(),
                type_parameters: None
            }),
            name: "change_observer".to_string(),
            passing: ArgumentPassing::Clone,
            construction: ArgumentConstruction::ExistingObject(),
        }
    }

    fn new_context_arg() -> Self {
        Self {
            arg_type: Box::new(TypeDescription::Type {
                name: "()".to_string(),
                type_parameters: None,
            }),
            name: "context".to_string(),
            passing: ArgumentPassing::Clone,
            construction: ArgumentConstruction::ExistingObject(),
        }
    }

    fn into_object_description(&self, type_name: &String , type_parameter_part: &String) -> ObjectDescription {
        ObjectDescription {
            type_name: type_name.clone(),
            type_parameter_part: type_parameter_part.clone(),
            //type_parameters: HashMap::new(),
            name: self.name.clone(),
            is_mutable: if let ArgumentPassing::MutableReference { .. } = self.passing {
                true
            } else {
                false
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectDescription {
    pub type_name: String,
    pub type_parameter_part: String,
    //pub type_parameters: HashMap<String, String>,
    pub name: String,
    pub is_mutable: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Constructor {
    New{function_name: Option<String>},
    NewWithObserver{function_name: Option<String>},
    NewWithObserverAndContext{function_name: Option<String>},
    NewWithArbitraryArgs{function_name: Option<String>, arguments: Vec<Argument>},
    FromJson,
    FromDefault,
    FromCode{code_template: String} 
}

impl Constructor {
   
    fn emit_fully_qualified_name(&self, name: &String, namespace: &Namespace, ignore: bool) -> String {
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
                    self.emit_fully_qualified_name(&arg.name, current_namespace, matches!(arg.construction,ArgumentConstruction::ExistingObject())),
                    arg.emit_postfix_code()
                )
            })
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn get_resolved_arg_type_parameters(
        &self,
        arg_type: &Box<TypeDescription>,
        already_resolved_tps: &HashMap<String, String>,
        resolved_tps: &mut HashMap<String, String>,
    ) {

        let mut type_params: &Option<Vec<Box<TypeDescription>>> = &None;

        match arg_type.as_ref() {
            TypeDescription::Type { type_parameters, ..} => {
                type_params = type_parameters;
            }
            
            TypeDescription::Generic { type_parameters, name } => {
                if let Some(resolved_name) = already_resolved_tps.get(name) {
                    resolved_tps.insert(name.clone(), resolved_name.clone());
                }
                type_params = type_parameters; 
            }
        }

        if let Some(params) = type_params {
            for param in params {
                self.get_resolved_arg_type_parameters(
                    &param,
                    already_resolved_tps,
                    resolved_tps,
                );
            }
        }

    }

    fn emit_arg_type_parameters_part_rec(
        &self,
        tp_part: &mut String,
        argument_type: &Box<TypeDescription>,
        resolved_type_parameters: &HashMap<String, String>,
    ) {
        let mut arg_type_params: &Option<Vec<Box<TypeDescription>>> = &None;

        match argument_type.as_ref() {

            TypeDescription::Type { name, type_parameters} => {
                tp_part.push_str(name);
                arg_type_params = type_parameters;
            }

            TypeDescription::Generic { name, type_parameters} => {
                if let Some(tn) = resolved_type_parameters.get(name) {
                    tp_part.push_str(tn);
                } else {
                    //TODO: Error Handling
                }
                arg_type_params = type_parameters;
            }            
        }

        if let Some(params) = arg_type_params {
            if !params.is_empty() {
                tp_part.push_str("<");

                for tp in params {
                    self.emit_arg_type_parameters_part_rec(tp_part, &tp, resolved_type_parameters);
                    tp_part.push_str(",")
                }

                tp_part.pop(); // pop last ,
                tp_part.push_str(">");
            }
        }
    }

    fn emit_arg_type_parameters_part(
        &self,
        arg_type_parameters: &Option<Vec<Box<TypeDescription>>>,
        type_parameters: &HashMap<String, String>,
    ) -> String {
                
        if let Some(arg_type_params) = arg_type_parameters {
           
            if !arg_type_params.is_empty() { 
                let mut tp_part = "<".to_string();

                for tp in arg_type_params {
                    self.emit_arg_type_parameters_part_rec(&mut tp_part, tp, type_parameters);
                    tp_part.push_str(",")
                }

                //tp_part.pop(); // pop last ,
                tp_part.push_str(">");
                return tp_part
            }
        }
        
        "".to_string()        
    }

    fn emit_arg_construction_code(
        &self,
        arg: &Argument,
        arg_constructor_name: String,
        pack_man: &PackageManager,
        current_namespace: &Namespace,
        type_parameters: &HashMap<String, String>,
    ) -> Result<String, Error> {


        match arg.arg_type.as_ref() {

            TypeDescription::Type {
                name,
                type_parameters: arg_type_parameters,
            } => {
                if let Some(type_desc) = pack_man.get_type(name) {
                    
                    if let Some(arg_constructor) = type_desc.constructors.get(&arg_constructor_name) {
   
                        let object_desc = arg.into_object_description(
                            &name, 
                            &self.emit_arg_type_parameters_part(arg_type_parameters, type_parameters), 
                        );

                        arg_constructor.emit_code_template(&object_desc, type_parameters, pack_man, current_namespace)
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

            TypeDescription::Generic { name , type_parameters: arg_type_parameters }=> {

                // get resolved type parameters.
                
                /*
                let mut type_parameters_for_arg_type = HashMap::<String, String>::new();
                self.get_resolved_arg_type_parameters(
                    &arg.arg_type,
                    type_parameters,
                    &mut type_parameters_for_arg_type,
                );
                 */
                

                // check if generic was already resolved. if so, try to get type and emit constructor code.
                // TODO: Think about what should happen if it is not yet resolved.
                if let Some(type_name) = type_parameters.get(name) {
                    if let Some(type_desc) = pack_man.get_type(&type_name) {    
                        if let Some(arg_constructor) = type_desc.constructors.get(&arg_constructor_name) {

                            let object_desc = arg.into_object_description(
                                &type_name, 
                                &self.emit_arg_type_parameters_part(arg_type_parameters, &type_parameters),
                            );

                            arg_constructor.emit_code_template(
                                &object_desc,
                                &type_parameters,
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
        type_parameters: &HashMap<String, String>,
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
                    type_parameters,
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

    fn emit_function_name(&self, function_name: &Option<String>) -> String {
        if let Some(func_name) = function_name {
            func_name.clone()
        } else {
            "new".into()
        }
    }

    fn emit_new_with_args(
        &self,
        od: &ObjectDescription,
        type_parameters: &HashMap<String, String>,
        function_name: &Option<String>,
        pack_man: &PackageManager,
        args: &Vec<Argument>,
        current_namespace: &Namespace,
    ) -> Result<String, Error> {
            
            let mut new_namespace = current_namespace.clone();
            new_namespace.add_part(&od.name);

            let args_construction_code = self.emit_args_construction_code(
                pack_man,
                args,
                &new_namespace,
                type_parameters,
            )?;

            Ok(format!(
                "{}\nlet{} {} = {}::{}{}({});",
                args_construction_code,
                self.emit_mutable(od.is_mutable),
                self.emit_fully_qualified_name(&od.name, current_namespace, false),
                od.type_name,
                if od.type_parameter_part.is_empty() {"".to_string() } else {od.type_parameter_part.clone() + "::"} ,
                self.emit_function_name(function_name),
                self.emit_args(args, &new_namespace)
            ))
        
    }
    

    fn emit_default(
        &self,
        od: &ObjectDescription,
        pack_man: &PackageManager,
        current_namespace: &Namespace,
    ) -> Result<String, Error> {

        Ok(format!(
            "let{} {}:{}{} = Default::default();",
            self.emit_mutable(od.is_mutable),
            self.emit_fully_qualified_name(&od.name, current_namespace, false),
            od.type_name,
            od.type_parameter_part
        ))       
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
        current_namespace: &Namespace,
    ) -> Result<String, Error> {
        let emit_fully_qualified_name = self.emit_fully_qualified_name(&od.name, current_namespace, false);

        Ok(format!(
            "let{} {}: {}{} = serde_json::from_value(data{}.clone()).expect(\"Could not create '{}' from Json.\");",
            self.emit_mutable(od.is_mutable),
            emit_fully_qualified_name,
            od.type_name,
            od.type_parameter_part,
            self.emit_json_path(current_namespace, od),
            emit_fully_qualified_name
        ))
    }

    fn emit_constructor_from_code(
        &self,
        od: &ObjectDescription,
        current_namespace: &Namespace,
        code_template: &String,
        type_parameters: &HashMap<String, String>,
    ) -> Result<String, Error> {

        let mut handlebars = Handlebars::new();
      
        handlebars.register_template_string("code", code_template)?;

        let mut data = HashMap::new();
        data.insert("fully_qualified_name".to_string(), self.emit_fully_qualified_name(&od.name, current_namespace, false));
        data.insert("type_name".to_string(), od.type_name.clone());
        data.insert("type_parameter_part".to_string(), od.type_parameter_part.clone());
        data.insert("mutable".to_string(), self.emit_mutable(od.is_mutable));
        for (param, resolved_param) in type_parameters {
            data.insert(format!("type_parameter_{}", param), resolved_param.clone());
        }

        match handlebars.render("code", &data) {
            Ok(res) => Ok(res),
            Err(err) => Err(err.into()) 
        }
    }

}

impl Constructor  {
    pub fn emit_code_template(
        &self,
        obj_desc: &ObjectDescription,
        type_parameters: &HashMap<String, String>,
        pack_man: &PackageManager,
        namespace: &Namespace,
    ) -> Result<String, Error> {

        

        match self {

            Self::New {function_name} => self.emit_new_with_args(obj_desc, type_parameters, function_name, pack_man, &vec![], namespace),

            Self::NewWithObserver{function_name} => self.emit_new_with_args(
                obj_desc,
                type_parameters,
                function_name,
                pack_man,
                &vec![Argument::new_change_observer_arg()],
                namespace,
            ),

            Self::NewWithObserverAndContext{function_name} => self.emit_new_with_args(
                obj_desc,
                type_parameters,
                function_name,
                pack_man,
                &vec![
                    Argument::new_change_observer_arg(),
                    Argument::new_context_arg(),
                ],
                namespace,
            ),

            Self::NewWithArbitraryArgs{function_name, arguments} => {
                self.emit_new_with_args(obj_desc, type_parameters, function_name, pack_man, arguments, namespace)
            }

            Self::FromJson => self.emit_new_from_json(obj_desc, namespace),

            Self::FromDefault => self.emit_default(obj_desc, pack_man, namespace),

            Self::FromCode{code_template} => self.emit_constructor_from_code(obj_desc, namespace, code_template, type_parameters)
            
        }
    }
}

#[test]
fn test_from_code() {

    let package_json = r#"
    {
        "name": "my_package",
        "version": "1.0.0",
        "crates": {
          "my_crate": {
            "types": {
              "MyType": {
                "type_parameters": ["U", "T"],
                "constructors":{
                    "New":{"NewWithObserver": {}},
                    "FromCode":{"FromCode":{"code_template": "let {{fully_qualified_name}}:{{type_parameter_U}} = 5;"}}
                 }
            }
            },
            "modules": {}
          }
        }
      }
    "#;

    let package_1: Package = serde_json::from_str(&package_json).expect("wrong format.");
    let mut pm_1 = PackageManager::new();
    pm_1.add_package(package_1);

    let t_1 = pm_1.get_type("my_crate::MyType").expect("msg");
    let c_1 = t_1.constructors.get("FromCode").expect("");
    let mut type_params_1 = HashMap::new();
    type_params_1.insert("U".to_string(), "i32".to_string());
    type_params_1.insert("T".to_string(), "i32".to_string());
    let mut ns_1: Namespace = Namespace::new();

    let obj_1 = ObjectDescription {
        type_name: "my_crate::MyType".to_string(),
        type_parameter_part: "".to_string(),
        name: "value".to_string(),
        is_mutable: false,
    };

    assert_eq!("let value:i32 = 5;", c_1.emit_code_template(&obj_1, &type_params_1, &pm_1, &ns_1).expect(""));    
}

#[test]
fn test_arbitrary_args() {
   
    let package_json = r#"
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
                       "value":{
                          "modules":{
                             
                          },
                          "types":{
                             "ValueType":{
                                "constructors":{
                                   "Json":"FromJson"
                                }
                             },
                             "ValueNode":{
                                "outputs":{
                                    "output": { 
                                        "type":{
                                            "Generic":{
                                                "name":"I"
                                            }
                                        }
                                    }
                                },
                                "type_parameters":[
                                   "I"
                                ],
                                "constructors":{
                                   "New":{
                                      "NewWithArbitraryArgs":{
                                         "arguments":[
                                            {
                                              "name":"value",

                                               "type":{
                                                  "Generic":{
                                                     "name":"I"
                                                  }
                                               },
                                               "passing":"Move",
                                               "construction":{
                                                  "Constructor":"Json"
                                               }
                                            },
                                            {
                                                "name":"change_observer",

                                               "type":{
                                                  "Type":{
                                                     "name":"()"
                                                  }
                                               },
                                               "passing":"Clone",
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
     }    
    "#;

    let package_1: Package = serde_json::from_str(&package_json).expect("wrong format.");
    let mut pm_1 = PackageManager::new();
    pm_1.add_package(package_1);

    let t_1 = pm_1.get_type("flowrs_std::nodes::value::ValueNode").expect("msg");
    let c_1 = t_1.constructors.get("New").expect("");
    let mut type_params_1 = HashMap::new();
    type_params_1.insert("I".to_string(), "i32".to_string());
    let mut ns_1: Namespace = Namespace::new();

    let obj_1 = ObjectDescription {
        type_name: "flowrs_std::nodes::value::ValueNode".to_string(),
        type_parameter_part: "".to_string(),
        name: "value_node".to_string(),
        is_mutable: false,
    };

    let code = c_1.emit_code_template(&obj_1, &type_params_1, &pm_1, &ns_1).expect("");

    println!("{}",code);

    assert_eq!(
        "let value_node_value: i32 = serde_json::from_value(data[\"value_node\"][\"value\"].clone()).expect(\"Could not create \'value_node_value\' from Json.\");\nlet value_node = flowrs_std::nodes::value::ValueNode::new(value_node_value, change_observer.clone());"
        , code)    
}

#[test]
fn test_arbitrary_args_with_vec() {
   
    let package_json = r#"
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
                       "value":{
                          "modules":{
                             
                          },
                          "types":{
                             "ValueType":{
                                "constructors":{
                                   "Json":"FromJson"
                                }
                             },
                             "ValueNode":{
                                "outputs":{
                                    "output": { 
                                        "type":{
                                            "Generic":{
                                                "name":"I"
                                            }
                                        }
                                    }
                                },
                                "type_parameters":[
                                   "I"
                                ],
                                "constructors":{
                                    "NewWithValue":{
                                        "NewWithArbitraryArgs":{
                                           "arguments":[
                                              {
                                                 "type":{
                                                    "Type":{
                                                       "name":"std::vec::Vec",
                                                       "type_parameters":[
                                                          {
                                                             "Generic":{
                                                                "name":"I"
                                                             }
                                                          }
                                                       ]
                                                    }
                                                 },
                                                 "name":"v",
                                                 "passing":"Move",
                                                 "construction":{
                                                    "Constructor":"Json"
                                                 }
                                              },
                                              {
                                                 "type":{
                                                    "Type":{
                                                       "name":"()"
                                                    }
                                                 },
                                                 "name":"change_observer",
                                                 "passing":"Clone",
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
     }    
    "#;

    let package_json_std = r#"
    {
        "name":"std",
        "version":"1.0.0",
        "crates":{
           "std":{
              "types":{
                 
              },
              "modules":{
                 "vec":{
                    "types":{
                       "Vec":{
                          "type_parameters":[
                             "I"
                          ],
                          "constructors":{
                             "Json":"FromJson"
                          }
                       },
                       "SimpleNode":{
                        "type_parameters":[
                           "I",
                           "O"
                        ],
                        "inputs":{
                           "input":{
                              "type":{
                                 "Generic":{
                                    "name":"I"
                                 }
                              }
                           }
                        },
                        "outputs":{
                           "output":{
                              "type":{
                                 "Generic":{
                                    "name":"O"
                                 }
                              }
                           }
                        },
                        "constructors":{     
                        }
                     }
                    },
                    "modules":{
                       
                    }
                 }
              }
           }
        }
     }
    "#;

 
    let package_1: Package = serde_json::from_str(&package_json).expect("wrong format.");
    let package_2: Package = serde_json::from_str(&package_json_std).expect("wrong format.");
    
    let mut pm = PackageManager::new();
    pm.add_package(package_1);
    pm.add_package(package_2);

    let t = pm.get_type("flowrs_std::nodes::value::ValueNode").expect("msg");
    let c = t.constructors.get("NewWithValue").expect("");
    let mut type_params = HashMap::new();
    type_params.insert("I".to_string(), "i32".to_string());
    let mut ns: Namespace = Namespace::new();

    let obj_1 = ObjectDescription {
        type_name: "flowrs_std::nodes::value::ValueNode".to_string(),
        type_parameter_part: "".to_string(),
        name: "value_node".to_string(),
        is_mutable: false,
    };

    let code = c.emit_code_template(&obj_1, &type_params, &pm, &ns).expect("");

    println!("{}",code);

   
    assert_eq!(        
        "let value_node_v: std::vec::Vec<i32,> = serde_json::from_value(data[\"value_node\"][\"v\"].clone()).expect(\"Could not create \'value_node_v\' from Json.\");\nlet value_node = flowrs_std::nodes::value::ValueNode::new(value_node_v, change_observer.clone());"
        , code)    
}