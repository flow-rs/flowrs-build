pub mod flow_model;
pub mod flow_project;
pub mod logging;

#[cfg(not(target_arch = "wasm32"))]
pub mod api {
    pub mod rest_handlers;
}
#[cfg(not(target_arch = "wasm32"))]
pub mod service {
    pub mod config;
    pub mod server;
}

pub mod runtime {
    #[cfg(target_arch = "wasm32")]
    pub mod in_memory_runner;
    #[cfg(not(target_arch = "wasm32"))]
    pub mod node_runtime;
    #[cfg(not(target_arch = "wasm32"))]
    pub mod orchestrator;

    pub mod runtime_args;
    pub mod runtime_constants;
}

#[cfg(target_arch = "wasm32")]
use crate::runtime::in_memory_runner::run_browser_flow;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    run_browser_flow();
}
