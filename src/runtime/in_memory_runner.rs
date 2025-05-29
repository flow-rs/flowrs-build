use anyhow::Error;
use std::panic;
use std::sync::Arc;

use flowrs::comm::communication::NodeCommunicator;
use flowrs::exec::execution::StandardExecutor;
use flowrs::exec::execution_configuration::ExecutionConfig;
use flowrs::flow::abstract_flow::Flow;
use flowrs::flow::flow_types::NodeIOIndex;
use flowrs::flow::flow_types::NodeId;
use flowrs::generate_local_connection;
use flowrs::node::ReceiveError;
use flowrs::nodes::node_io::TypedInput;
use flowrs::nodes::node_io::TypedOutput;
use flowrs::sched::scheduling_config::SchedulingConfig;
use flowrs::types::type_registry::PollFn;
use flowrs::types::type_registry::POLL_REGISTRY;
use flowrs::types::type_registry::TYPE_REGISTRY;
use std::any::TypeId;

use flowrs::comm::communication::Communicator;
use flowrs::node::Node;
use flowrs::nodes::node_io::SetupIO;
use flowrs::types::type_registry::register_flush_fn;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Builder;

/// Dummy flow setup — to be replaced with real flow loader
async fn return_dummy_flow() -> Result<Flow, Error> {
    use flowrs::connect_nodes;
    use flowrs_std::add::SimpleAddNode;
    use flowrs_std::debug::DebugNode;
    use flowrs_std::value::ValueNode;

    let mut flow = Flow::new_empty();

    let number_node_1 = ValueNode::<u32>::new(3);
    let number_node_2 = ValueNode::<u32>::new(2);
    let add_node = SimpleAddNode::<u32>::new();
    let debug_node = DebugNode::<u32>::new(true);

    flow.add_node_with_id(Box::new(number_node_1), 1);
    flow.add_node_with_id(Box::new(number_node_2), 2);
    flow.add_node_with_id(Box::new(add_node), 3);
    flow.add_node_with_id(Box::new(debug_node), 4);

    connect_nodes!(u32, flow, 1, 3, 0, 0)?;
    connect_nodes!(u32, flow, 2, 3, 0, 1)?;
    connect_nodes!(u32, flow, 3, 4, 0, 0)?;

    Ok(flow)
}

/// Dummy scheduling logic — 1 runtime only
fn dummy_scheduling(flow: &Flow) -> SchedulingConfig {
    let mut config = SchedulingConfig::new();
    let mut nodes: Vec<NodeId> = flow.get_nodes().map(|(id, _)| *id).collect();
    nodes.sort();
    for node_id in nodes {
        config.assign_node(0, node_id);
    }
    config
}

static INIT: std::sync::Once = std::sync::Once::new();

fn init_tracing() {
    INIT.call_once(|| {
        tracing_wasm::set_as_global_default();
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_browser_flow() {
    // needed to use web console
    console_error_panic_hook::set_once();
    // Init tracing to log to browser console
    init_tracing();
    spawn_local(async {
        if let Err(e) = in_memory_main().await {
            console::error_1(&format!("Flow error: {}", e).into());
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async {
            if let Err(e) = in_memory_main().await {
                eprintln!("Flow error: {}", e);
            }
        });
}

pub async fn in_memory_main() -> Result<(), Error> {
    tracing::info!("Entering in_memory_main...");
    let flow = return_dummy_flow().await?;
    let scheduling_config = dummy_scheduling(&flow);
    let exec_config = ExecutionConfig::from_scheduling_config(&scheduling_config, 0);

    let mut executor = StandardExecutor::new();
    let abstract_flow = Arc::new(tokio::sync::Mutex::new(flow));
    executor
        .initialize_nodes(Arc::clone(&abstract_flow), &exec_config)
        .await?;

    {
        let flow_guard = abstract_flow.lock().await;
        let mut registry = TYPE_REGISTRY.lock().await;

        for conn in flow_guard.get_connections() {
            let (type_name, type_id) = flow_guard
                .get_connection_type(conn)
                .ok_or_else(|| anyhow::anyhow!("Missing type info for connection"))?;

            let conn_fn = registry
                .get(type_id)
                .ok_or_else(|| anyhow::anyhow!("No connection function for type {:?}", type_id))?;

            let mut sender_guard = executor
                .execution_nodes
                .get(&conn.sender_id)
                .ok_or_else(|| anyhow::anyhow!("Missing sender node {}", conn.sender_id))?
                .lock()
                .await;

            let mut receiver_guard = executor
                .execution_nodes
                .get(&conn.receiver_id)
                .ok_or_else(|| anyhow::anyhow!("Missing receiver node {}", conn.receiver_id))?
                .lock()
                .await;

            conn_fn(
                conn.sender_id,
                conn.receiver_id,
                conn.send_out_idx,
                conn.recv_in_idx,
                sender_guard.get_io_mut(),
                receiver_guard.get_io_mut(),
            );
        }
    }

    executor.ready_nodes().await?;
    executor.start_execution().await;

    Ok(())
}
