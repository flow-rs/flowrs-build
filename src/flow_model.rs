use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use syn::Ident;

use std::path::PathBuf;
use std::process::Command;

use crate::package::{Constructor, Namespace, ObjectDescription, Package};
use crate::package_manager::PackageManager;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ConnectionModel {
    input_node: String,
    output_node: String,
    input: String,
    output: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeModel {
    node_type: String,
    type_parameters: HashMap<String, String>,
    constructor: String
    //inputs: HashMap<String, InputModel>,
    //outputs: HashMap<String, OutputModel>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowModel {
    nodes: HashMap<String, NodeModel>,
    connections: Vec<ConnectionModel>,
    data: Value,
}

pub trait CodeEmitter {
    fn emit_flow_code(&self, flow: &FlowModel, pm: &PackageManager) -> String;
}

pub struct StandardCodeEmitter {}

impl StandardCodeEmitter {
    fn emit_function(&self, body: &TokenStream) -> TokenStream {
        quote! {
            #[wasm_bindgen]
            pub fn run() {
                #body
            }
        }
    }

    fn emit_function_body(&self, flow: &FlowModel, pm: &PackageManager) -> TokenStream {
        let mut body = TokenStream::new();

        self.emit_std_locals(&mut body);

        self.emit_nodes(flow, &mut body, pm);

        self.emit_node_connections(flow, &mut body);

        self.emit_flow(flow, &mut body);

        self.emit_exec_call(&mut body);

        body
    }

    fn emit_nodes(&self, flow: &FlowModel, tokens: &mut TokenStream, pm: &PackageManager) {
        for (node_name, node) in &flow.nodes {
            let generated_code = self.emit_node(node_name, node, pm);
            tokens.extend(generated_code);
        }
    }

    fn emit_node_connections(&self, flow: &FlowModel, tokens: &mut TokenStream) {
        for connection in &flow.connections {
            let generated_code = self.emit_node_connection(connection);
            tokens.extend(generated_code);
        }
    }

    fn node_model_to_object(&self, node_name: &String, node: &NodeModel) -> ObjectDescription {
        ObjectDescription {
            name: node_name.clone(),
            type_name: node.node_type.clone(),
            type_parameters: node.type_parameters.clone(),
            is_mutable: false,
        }
    }

    fn emit_node(&self, node_name: &str, node: &NodeModel, pm: &PackageManager) -> TokenStream {
        if let Some(node_type) = pm.get_type(&node.node_type) {

            if let Some(constructor) = node_type.constructors.get(&node.constructor) {

                if let Ok(code) = constructor.emit_code_template(
                    &self.node_model_to_object(&node_name.to_string(), node),
                    pm,
                    &Namespace::new(),
                ) {
                    let tok: TokenStream = code.parse().unwrap();

                    return quote! {
                        #tok
                    };
                } else {
                    //TODO: Error reporting.
                }
            } else {
                // TODO: Error reporting.
            }
        }
        quote! {}
    }

    fn emit_node_connection(&self, connection: &ConnectionModel) -> TokenStream {
        let node_out_ident = Ident::new(&connection.input_node, proc_macro2::Span::call_site());
        let node_inp_ident = Ident::new(&connection.output_node, proc_macro2::Span::call_site());
        let output_ident = Ident::new(&connection.output, proc_macro2::Span::call_site());
        let input_ident = Ident::new(&connection.input, proc_macro2::Span::call_site());

        quote! {
            connect(#node_out_ident.#output_ident.clone(), #node_inp_ident.#input_ident.clone());
        }
    }

    fn emit_std_locals(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            let co = ChangeObserver::new();
            let change_observer = Some(&co);
            let mut file = File::open("flow-project.json").expect("Failed to open flow project file.");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Failed to read flow project file.");
            let data: Value = from_reader(contents.as_bytes()).expect("Failed to parse flow project file.");
        });
    }

    fn emit_flow(&self, flow: &FlowModel, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            let mut flow = Flow::new_empty("wasm", Version::new(0, 0, 1));
        });

        let mut id: u128 = 0;
        for (node_name, node) in &flow.nodes {
            let node_ident = Ident::new(&node_name, proc_macro2::Span::call_site());
            let node_type = node.node_type.clone();
            tokens.extend(quote! {
                flow.add_node_with_id_and_desc(
                    #node_ident,
                    #id,
                    NodeDescription {name: #node_name.into(), description: #node_name.into() /*TODO*/, kind: #node_type.into()});
            });
            id += 1;
        }
    }

    fn emit_use_decls(&self) -> TokenStream {
        quote! {
            use wasm_bindgen::prelude::*;

            use serde_json::{from_reader, from_value, Value};
            use std::fs::File;
            use std::io::Read;

            use flowrs::nodes::node_description::NodeDescription;
            use flowrs::nodes::node::ChangeObserver;
            use flowrs::nodes::connection::connect;

            use flowrs::flow::version::Version;
            use flowrs::flow::flow::Flow;

            use flowrs::exec::execution::{Executor, StandardExecutor};
            use flowrs::exec::node_updater::SingleThreadedNodeUpdater;
            use flowrs::sched::round_robin::RoundRobinScheduler;
        }
    }

    fn emit_exec_call(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            let node_updater = SingleThreadedNodeUpdater::new(None);
            let scheduler = RoundRobinScheduler::new();
            let mut executor = StandardExecutor::new(co);
            let _ = executor.run(flow, scheduler, node_updater);
        });
    }
}

impl CodeEmitter for StandardCodeEmitter {
    fn emit_flow_code(&self, flow: &FlowModel, pm: &PackageManager) -> String {
        format!(
            "{}{}",
            self.emit_use_decls(),
            self.emit_function(&self.emit_function_body(flow, pm))
        )
    }
}

#[test]
fn test() {
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
                                "constructor":"NewWithObserver"
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

    let flow_json = r#"
    {
        "nodes": {
            "node1": {
                "node_type": "flowrs_std::nodes::debug::DebugNode",
                "type_parameters": {"I": "i32"}

            },
            "node2": {
                "node_type": "flowrs_std::nods::debug::DebugNode",
                "type_parameters": {"I": "i32"}
            }
        },
        "connections": [
            {
                "input_node": "node1",
                "output_node": "node2",
                "input": "input",
                "output": "output"
            }
        ]
    }
    "#;

    let flow_model: FlowModel = serde_json::from_str(&flow_json).expect("wrong format.");

    let mut pm = PackageManager::new();

    let p: Package = serde_json::from_str(package_json).expect("format wrong.");

    pm.add_package(p);

    let rce = StandardCodeEmitter {};
    println!("{}", rce.emit_flow_code(&flow_model, &pm));

    //let pack = StandardWasmPackager::new(rce);
    //pack.compile_package(&flow_model);
}
