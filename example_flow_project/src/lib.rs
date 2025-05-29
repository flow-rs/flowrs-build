mod flows;

use anyhow::{Error, Result};
use flowrs::flow::abstract_flow::Flow;
use flowrs::Flow;
use flows::addition;

/// Native version: returns the constructed flow
#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub async fn load_flow(name: &str) -> Result<Flow, Error> {
    match name {
        "addition" => addition::return_flow().await,
        _ => Err(Error::msg(format!("Unknown flow name: {}", name))),
    }
}

/// WebAssembly version: returns success or error string, but doesn't expose the flow directly
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn load_flow(name: &str) -> Result<(), JsValue> {
    match name {
        "addition" => {
            let _ = addition::return_flow()
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            // You could call .run() here if you want, or return structured data later
            Ok(())
        }
        _ => Err(JsValue::from_str(&format!("Unknown flow name: {}", name))),
    }
}
