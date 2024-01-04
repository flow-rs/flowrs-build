use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use syn::Ident;
use crate::package::{Namespace, ObjectDescription, Package};
use crate::package_manager::PackageManager;

use anyhow::{Error, Result};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct ConnectionModel {
    from_node: String,
    to_node: String,
    to_input: String,
    from_output: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NodeModel {
    node_type: String,
    type_parameters: HashMap<String, String>,
    constructor: String
    //inputs: HashMap<String, InputModel>,
    //outputs: HashMap<String, OutputModel>
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FlowModel {
    nodes: HashMap<String, NodeModel>,
    connections: Vec<ConnectionModel>,
    data: Value,
}

pub trait CodeEmitter {
    fn emit_flow_code(&self, flow: &FlowModel, pm: &PackageManager) -> Result<String, Error>;
}

pub struct StandardCodeEmitter {}

impl StandardCodeEmitter {
    fn emit_functions(&self, init_function_body: &TokenStream) -> TokenStream {
        quote! {

            #[cfg(target_arch = "wasm32")]
            #[wasm_bindgen]
            extern "C" {
                # [wasm_bindgen (js_namespace = console)]
                fn log(s: &str);
            }
            #[cfg(target_arch = "wasm32")]
            macro_rules ! println { ($ ($ t : tt) *) => { log (format ! ($ ($ t) *) . as_str ()) ; } }
            #[cfg(target_arch = "wasm32")]
            #[wasm_bindgen]
            pub fn wasm_run() {
                
                let mut ctx = Box::new(init());

                let node_updater = SingleThreadedNodeUpdater::new(None);
                let scheduler = RoundRobinScheduler::new();

                let res = ctx.executor.run(ctx.flow, scheduler, node_updater);
            }

            #[cfg(not(target_arch = "wasm32"))]
            #[no_mangle]
            pub extern "C" fn native_init() -> *mut ExecutionContextHandle {
                let ctx = Box::new(init());
                Box::into_raw(ctx).cast()
            }

            #[cfg(not(target_arch = "wasm32"))]
            #[no_mangle]
            pub extern "C" fn native_run(num_workers: usize, ctx_handle: *mut ExecutionContextHandle) -> *const c_char {
                let mut ctx = unsafe { Box::from_raw(ctx_handle.cast::<ExecutionContext>()) };
                
                let node_updater = MultiThreadedNodeUpdater::new(num_workers);
                let scheduler = RoundRobinScheduler::new();

                let res = ctx.executor.run(ctx.flow, scheduler, node_updater);

                CString::new(format!("{:?}", res)).expect("Cannot convert result to a C-String.").into_raw()
            }
            #[no_mangle]
            pub unsafe extern fn native_free_string(ptr: *const c_char) {
                let _ = CString::from_raw(ptr as *mut _);
            }

            #[cfg(not(target_arch = "wasm32"))]
            #[no_mangle]
            pub extern "C" fn native_cancel(ctx_handle: *mut ExecutionContextHandle) {
                let ctx = unsafe { Box::from_raw(ctx_handle.cast::<ExecutionContext>()) };
                ctx.executor.controller().lock().unwrap().cancel()
            }


            pub fn init() -> ExecutionContext {
                #init_function_body
            }
        }
    }

    fn emit_init_function_body(&self, flow: &FlowModel, pm: &PackageManager) -> Result<TokenStream, Error> {
        let mut body = TokenStream::new();

        self.emit_std_locals(&mut body, flow);

        self.emit_nodes(flow, &mut body, pm)?;

        self.emit_node_connections(flow, &mut body);

        self.emit_flow(flow, &mut body);

        self.emit_context_creation(&mut body);

        Ok(body)
    }

    fn emit_nodes(&self, flow: &FlowModel, tokens: &mut TokenStream, pm: &PackageManager) -> Result<(), Error> {
        for (node_name, node) in &flow.nodes {
            let generated_code = self.emit_node(node_name, node, pm)?;
            tokens.extend(generated_code);
        }
        Ok(())
    }

    fn emit_node_connections(&self, flow: &FlowModel, tokens: &mut TokenStream) {
        for connection in &flow.connections {
            let generated_code = self.emit_node_connection(connection);
            tokens.extend(generated_code);
        }
    }

    fn node_model_to_object(&self, node_name: &String, node: &NodeModel, pm: &PackageManager) -> ObjectDescription {
        ObjectDescription {
            name: node_name.clone(),
            type_name: node.node_type.clone(),
            type_parameter_part: self.emit_type_parameter_part(&node, pm),
            is_mutable: false,
        }
    }

    fn emit_type_parameter_part(&self, node: &NodeModel, pm: &PackageManager) -> String {
        let mut tp_part = "".to_string();
        if let Some(t) = pm.get_type(&node.node_type) {
            if let Some(tp) = &t.type_parameters {
                self.emit_type_parameter_part_rec(&tp, &node.type_parameters, pm, &mut tp_part);
            }
        }
        tp_part 
    }

    fn emit_type_parameter_part_rec(&self, type_parameters: &Vec<String>, resolved_type_parameters: &HashMap<String, String>, pm: &PackageManager, tp_part: &mut String) {
        
        if !type_parameters.is_empty() {
            tp_part.push_str("<");
        }

        for type_parameter in type_parameters {

            if let Some(type_name) = resolved_type_parameters.get(type_parameter) {

                tp_part.push_str(type_name);
                if let Some(t) = pm.get_type(type_name) {
                    if let Some(tps) = &t.type_parameters {
                        self.emit_type_parameter_part_rec(tps, resolved_type_parameters, pm, tp_part);  
                    }      
                } 
                tp_part.push_str(",");
            } else {
                //TODO: Error Reporting. 
            }
        }
        if !type_parameters.is_empty() {
            //tp_part.pop(); // pop last ,
            tp_part.push_str(">");
        }
    }

    fn emit_node(&self, node_name: &str, node: &NodeModel, pm: &PackageManager) -> Result<TokenStream, Error>  {
        if let Some(node_type) = pm.get_type(&node.node_type) {

            if let Some(constructor) = node_type.constructors.get(&node.constructor) {
                
                let res = constructor.emit_code_template(
                    &self.node_model_to_object(&node_name.to_string(), node, pm),
                    &node.type_parameters,
                    pm,
                    &Namespace::new(),
                );

                match res {
                    Ok(code)=> { 
                        let tok: TokenStream = code.parse().unwrap();
                        Ok(quote! {
                            #tok
                        }) 
                    },
                    Err(err) => {
                        Err(err)
                    }
                }
            } else {
                Err(anyhow::Error::msg(format!("Cannot find constructor '{}' for node '{}' with type '{}'", node.constructor, node_name, node.node_type)))
            }
        } else {
            Err(anyhow::Error::msg(format!("Cannot find type '{}' for node '{}'.", node.node_type, node_name)))
        }
       
    }

    fn emit_node_connection(&self, connection: &ConnectionModel) -> TokenStream {
        let node_out_ident = Ident::new(&connection.from_node, proc_macro2::Span::call_site());
        let node_inp_ident = Ident::new(&connection.to_node, proc_macro2::Span::call_site());
        let output_ident = Ident::new(&connection.from_output, proc_macro2::Span::call_site());
        let input_ident = Ident::new(&connection.to_input, proc_macro2::Span::call_site());

        quote! {
            connect(#node_out_ident.#output_ident.clone(), #node_inp_ident.#input_ident.clone());
        }
    }

    fn emit_std_locals(&self, tokens: &mut TokenStream, flow: &FlowModel) {

        let data_str = serde_json::to_string(&flow.data).unwrap();

        tokens.extend(quote! {
            let co = ChangeObserver::new();
            let change_observer = Some(&co);
            let context = Arc::new(Mutex::new(Context::new()));
            let data_str = #data_str;
            let data: Value = serde_json::from_str(&data_str).expect("Failed to parse flow project data.");
        });
    }

    fn emit_flow(&self, flow: &FlowModel, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            let mut flow = Flow::new_empty();
        });

        let mut id: u128 = 0;
        for (node_name, node) in &flow.nodes {
            let node_ident = Ident::new(&node_name, proc_macro2::Span::call_site());
            let node_type = node.node_type.clone();
            tokens.extend(quote! {
                flow.add_node_with_id_and_desc(
                    #node_ident,
                    #id,
                    NodeDescription {name: #node_name.into(), description: #node_name.into() /*TODO: get a node desc.*/, kind: #node_type.into()});
            });
            id += 1;
        }
    }

    fn emit_use_decls(&self) -> TokenStream {
        quote! {
            use flowrs::exec::execution::{Executor, StandardExecutor, ExecutionContext, ExecutionContextHandle};
            use flowrs::exec::node_updater::{NodeUpdater, SingleThreadedNodeUpdater, MultiThreadedNodeUpdater};
            use flowrs::flow::flow::Flow;
            use flowrs::nodes::connection::connect;
            use flowrs::nodes::node::{ChangeObserver, Context};
            use flowrs::nodes::node_description::NodeDescription;
            use flowrs::sched::{scheduler::Scheduler, round_robin::RoundRobinScheduler};
            use serde_json::Value;
            use std::sync::{Arc, Mutex};
            use std::ffi::{CString, CStr};
            use std::os::raw::c_char;
            use wasm_bindgen::prelude::*;
        }
    }

    fn emit_context_creation(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            let executor = StandardExecutor::new(co);
            ExecutionContext::new(executor, flow)
        });
    }
}

impl CodeEmitter for StandardCodeEmitter {
    fn emit_flow_code(&self, flow: &FlowModel, pm: &PackageManager) -> Result<String, Error> {
        
        let init_function_body = self.emit_init_function_body(flow, pm)?;
        
        Ok(format!(
            "{}{}",
            self.emit_use_decls(),
            self.emit_functions(&init_function_body)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const PACKAGE_JSON: &str = r#"
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
                                 "NewWithObserver":{
                                    
                                 }
                              }
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
                              "Json":"FromJson"
                           }
                        },
                        "ValueNode":{
                           "outputs":{
                              "output":{
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
                                          "type":{
                                             "Generic":{
                                                "name":"I"
                                             }
                                          },
                                          "name":"value",
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
                  },
                  "timer":{
                     "modules":{
                        
                     },
                     "types":{
                        "TimerNodeConfig":{
                           "constructors":{
                              "Json":"FromJson"
                           }
                        },
                        "PollTimer":{
                           "type_parameters":[
                              "U"
                           ],
                           "constructors":{
                              "New":{
                                 "New":{
                                    
                                 }
                              }
                           }
                        },
                        "SelectedTimer":{
                           "type_parameters":[
                              "U"
                           ],
                           "constructors":{
                              "New":{
                                 "New":{
                                    
                                 }
                              }
                           }
                        },
                        "TimerNode":{
                           "inputs":{
                              "config_input":{
                                 "type":{
                                    "Type":{
                                       "name":"flowrs_std::nodes::timer::TimerNodeConfig"
                                    }
                                 }
                              },
                              "token_input":{
                                 "type":{
                                    "Generic":{
                                       "name":"U"
                                    }
                                 }
                              }
                           },
                           "outputs":{
                              "token_output":{
                                 "type":{
                                    "Generic":{
                                       "name":"U"
                                    }
                                 }
                              }
                           },
                           "type_parameters":[
                              "T",
                              "U"
                           ],
                           "constructors":{
                              "NewWithToken":{
                                 "NewWithArbitraryArgs":{
                                    "function_name":"new_with_token",
                                    "arguments":[
                                       {
                                          "type":{
                                             "Generic":{
                                                "name":"T",
                                                "type_parameters":[
                                                   {
                                                      "Generic":{
                                                         "name":"U"
                                                      }
                                                   }
                                                ]
                                             }
                                          },
                                          "name":"timer",
                                          "passing":"Move",
                                          "construction":{
                                             "Constructor":"New"
                                          }
                                       },
                                       {
                                          "type":{
                                             "Generic":{
                                                "name":"U"
                                             }
                                          },
                                          "name":"token_object",
                                          "passing":"Move",
                                          "construction":{
                                             "Constructor":"New"
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
                              },
                              "New":{
                                 "NewWithArbitraryArgs":{
                                    "arguments":[
                                       {
                                          "type":{
                                             "Generic":{
                                                "name":"T",
                                                "type_parameters":[
                                                   {
                                                      "Generic":{
                                                         "name":"U"
                                                      }
                                                   }
                                                ]
                                             }
                                          },
                                          "name":"timer",
                                          "passing":"Move",
                                          "construction":{
                                             "Constructor":"New"
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

    const FLOW_JSON: &str = r#"
{
    "nodes": {
        "debug_node": {
            "node_type": "flowrs_std::nodes::debug::DebugNode",
            "type_parameters": {"I": "i32"},
            "constructor": "New"

        },
        "timer_config_node": {
            "node_type": "flowrs_std::nodes::value::ValueNode",
            "type_parameters": {"I": "flowrs_std::nodes::timer::TimerNodeConfig"},
            "constructor": "New"

        },
        "timer_token_node": {
            "node_type": "flowrs_std::nodes::value::ValueNode",
            "type_parameters": {"I": "i32"},
            "constructor": "New"

        },
         "timer_node": {
            "node_type": "flowrs_std::nodes::timer::TimerNode",
            "type_parameters": {"T": "flowrs_std::nodes::timer::SelectedTimer", "U": "i32"},
            "constructor": "New"
        }
    },
    "connections": [
        {
            "from_node": "timer_config_node",
            "from_output": "output",
            "to_node": "timer_node",
            "to_input": "config_input"
        },
        {
            "from_node": "timer_token_node",
            "from_output": "output",
            "to_node": "timer_node",
            "to_input": "token_input"
        },
        {
            "from_node": "timer_node",
            "from_output": "token_output",
            "to_node": "debug_node",
            "to_input": "input"
        }
    ],
    "data" : {
        "timer_config_node": {
            "value": {"duration": {"secs": 1, "nanos": 0}}
        },
         "timer_token_node": {
            "value": 42
        }
    }
}
    "#;
    #[test]
    fn test() {

        let flow_model: FlowModel = serde_json::from_str(&FLOW_JSON).expect("wrong format.");

        let mut pm = PackageManager::new();

        let p: Package = serde_json::from_str(PACKAGE_JSON).expect("format wrong.");

        pm.add_package(p);

        let rce = StandardCodeEmitter {};
        println!("{}", rce.emit_flow_code(&flow_model, &pm).expect("flow code wrong."));

        //let pack = StandardWasmPackager::new(rce);
        //pack.compile_package(&flow_model);
    }
}
