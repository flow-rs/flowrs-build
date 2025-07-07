use flowrs::exec::execution::{
    ExecutionContext, ExecutionContextHandle, Executor, StandardExecutor,
};
use flowrs::exec::node_updater::{
    MultiThreadedNodeUpdater, NodeUpdater, SingleThreadedNodeUpdater,
};
use flowrs::flow::flow::Flow;
use flowrs::nodes::connection::connect;
use flowrs::nodes::node::{ChangeObserver, Context};
use flowrs::nodes::node_description::NodeDescription;
use flowrs::sched::{round_robin::RoundRobinScheduler, scheduler::Scheduler};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
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
pub extern "C" fn native_run(
    num_workers: usize,
    ctx_handle: *mut ExecutionContextHandle,
) -> *const c_char {
    let mut ctx = unsafe { Box::from_raw(ctx_handle.cast::<ExecutionContext>()) };
    let node_updater = MultiThreadedNodeUpdater::new(num_workers);
    let scheduler = RoundRobinScheduler::new();
    let res = ctx.executor.run(ctx.flow, scheduler, node_updater);
    CString::new(format!("{:?}", res))
        .expect("Cannot convert result to a C-String.")
        .into_raw()
}
#[no_mangle]
pub unsafe extern "C" fn native_free_string(ptr: *const c_char) {
    let _ = CString::from_raw(ptr as *mut _);
}
#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub extern "C" fn native_cancel(ctx_handle: *mut ExecutionContextHandle) {
    let ctx = unsafe { Box::from_raw(ctx_handle.cast::<ExecutionContext>()) };
    ctx.executor.controller().lock().unwrap().cancel()
}
pub fn init() -> ExecutionContext {
    let co = ChangeObserver::new();
    let change_observer = Some(&co);
    let context = Arc::new(Mutex::new(Context::new()));
    let data_str = "{\"value_node_one\":{\"value\":2},\"value_node_two\":{\"value\":3}}";
    let data: Value = serde_json::from_str(&data_str).expect("Failed to parse flow project data.");
    let add_node =
        flowrs_std::nodes::binops::add::AddNode::<i32, i32, i32>::new(change_observer.clone());
    let debug_node = flowrs_std::nodes::debug::DebugNode::<i32>::new(change_observer.clone());
    let value_node_one_value: i32 = serde_json::from_value(data["value_node_one"]["value"].clone())
        .expect("Could not create 'value_node_one_value' from Json.");
    let value_node_one = flowrs_std::nodes::value::ValueNode::<i32>::new(
        value_node_one_value,
        change_observer.clone(),
    );
    let value_node_two_value: i32 = serde_json::from_value(data["value_node_two"]["value"].clone())
        .expect("Could not create 'value_node_two_value' from Json.");
    let value_node_two = flowrs_std::nodes::value::ValueNode::<i32>::new(
        value_node_two_value,
        change_observer.clone(),
    );
    connect(value_node_one.output.clone(), add_node.input_1.clone());
    connect(value_node_two.output.clone(), add_node.input_2.clone());
    connect(add_node.output_1.clone(), debug_node.input.clone());
    let mut flow = Flow::new_empty();
    flow.add_node_with_id_and_desc(
        add_node,
        0u128,
        NodeDescription {
            name: "add_node".into(),
            description: "add_node".into(),
            kind: "flowrs_std::nodes::binops::add::AddNode".into(),
        },
    );
    flow.add_node_with_id_and_desc(
        debug_node,
        1u128,
        NodeDescription {
            name: "debug_node".into(),
            description: "debug_node".into(),
            kind: "flowrs_std::nodes::debug::DebugNode".into(),
        },
    );
    flow.add_node_with_id_and_desc(
        value_node_one,
        2u128,
        NodeDescription {
            name: "value_node_one".into(),
            description: "value_node_one".into(),
            kind: "flowrs_std::nodes::value::ValueNode".into(),
        },
    );
    flow.add_node_with_id_and_desc(
        value_node_two,
        3u128,
        NodeDescription {
            name: "value_node_two".into(),
            description: "value_node_two".into(),
            kind: "flowrs_std::nodes::value::ValueNode".into(),
        },
    );
    let executor = StandardExecutor::new(co);
    ExecutionContext::new(executor, flow)
}
