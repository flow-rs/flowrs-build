use anyhow::Error;
use clap::Parser;
use flowrs::connect_nodes;
use flowrs::exec::execution_configuration::ExecutionConfig;
use flowrs::flow::flow::Flow;
use flowrs::flow::flow_types::NodeId;
use flowrs::generate_local_connection;
use flowrs::sched::scheduling_config::RuntimeId;
use flowrs::sched::scheduling_config::SchedulingConfig;
use flowrs_build::logging;
use flowrs_build::logging::print_startup_banner;
use flowrs_build::runtime::node_runtime::NodeRuntime;
use flowrs_build::runtime::orchestrator::Orchestrator;
use flowrs_build::runtime::runtime_args::Arguments;
#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use tokio::net::lookup_host;

use flowrs::comm::communication::Communicator;
use flowrs::comm::communication::NodeCommunicator;
use flowrs::flow::flow_types::NodeIOIndex;
use flowrs::node::ReceiveError;
use flowrs::nodes::node_io::SetupIO;
use flowrs::nodes::node_io::TypedInput;
use flowrs::nodes::node_io::TypedOutput;
use flowrs::types::type_registry::register_flush_fn;
use flowrs::types::type_registry::PollFn;
use flowrs::types::type_registry::POLL_REGISTRY;
use flowrs::types::type_registry::TYPE_REGISTRY;
use std::any::TypeId;

#[cfg(not(target_arch = "wasm32"))]
async fn get_orchestrator_address() -> Result<SocketAddr, anyhow::Error> {
    let mut addrs = lookup_host("orchestrator:5000").await?;
    addrs
        .next()
        .ok_or_else(|| anyhow::Error::msg("No valid IP found for orchestrator"))
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::init_logging();
    print_startup_banner();
    // Define the CLI application using clap
    let args = Arguments::parse();

    tracing::debug!("Create Dummy Flow");

    // Step 1: Create the flow definition
    let abstract_flow = return_dummy_flow().await?;

    tracing::debug!("Create Dummy Scheduling");
    // Step 2: Generate the scheduling configuration (Global)
    let num_runtimes = 2; // Hardcoded for now, later can be dynamically set
    let scheduling_config = dummy_scheduling(&abstract_flow, num_runtimes);

    // Step 3: Get the orchestrator's address
    let orchestrator_addr = get_orchestrator_address().await?;

    tracing::debug!(
        "Orchestrator Address: {}",
        orchestrator_addr.ip().to_string()
    );

    // Step 4: Determine role and start the appropriate component
    match args.role.as_str() {
        "orchestrator" => {
            //orchestrator = id 0
            let execution_config = ExecutionConfig::from_scheduling_config(&scheduling_config, 0);
            tracing::debug!("[Orchestrator] ExecutionConfig: {:?}", execution_config);
            let orchestrator = Arc::new(Orchestrator::new(abstract_flow, execution_config).await?);
            orchestrator.run(args, scheduling_config).await?;
        }
        "node-runtime" => {
            let runtime_id: RuntimeId = args.runtime_id.expect("Missing runtime ID");
            let execution_config =
                ExecutionConfig::from_scheduling_config(&scheduling_config, runtime_id);
            tracing::debug!("[Node RT] ExecutionConfig: {:?}", execution_config);
            let mut node_runtime = NodeRuntime::new(
                orchestrator_addr.ip().to_string(),
                runtime_id,
                abstract_flow,
                execution_config,
            )
            .await
            .expect("should construct");
            node_runtime.run(args, orchestrator_addr).await?;
        }
        _ => {
            eprintln!("Invalid role specified. Use 'orchestrator' or 'node-runtime'.");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn return_dummy_flow() -> Result<Flow, Error> {
    use flowrs_std::add::SimpleAddNode;
    use flowrs_std::debug::DebugNode;
    use flowrs_std::value::ValueNode;

    let mut flow = Flow::new_empty();

    // Define and add nodes
    let number_node_1 = ValueNode::<u32>::new(3);
    let number_node_2 = ValueNode::<u32>::new(2);
    let add_node = SimpleAddNode::<u32>::new();
    let warn_if_no_messages = true;
    let debug_node = DebugNode::<u32>::new(warn_if_no_messages);

    flow.add_node_with_id(Box::new(number_node_1), 1);
    flow.add_node_with_id(Box::new(number_node_2), 2);
    flow.add_node_with_id(Box::new(add_node), 3);
    flow.add_node_with_id(Box::new(debug_node), 4);

    // Register type + create connection functions + connect nodes
    connect_nodes!(u32, flow, 1, 3, 0, 0)?;
    connect_nodes!(u32, flow, 2, 3, 0, 1)?;
    connect_nodes!(u32, flow, 3, 4, 0, 0)?;

    Ok(flow)
}

#[cfg(not(target_arch = "wasm32"))]
fn dummy_scheduling(abstract_flow: &Flow, num_runtimes: u128) -> SchedulingConfig {
    let mut scheduling_config = SchedulingConfig::new();

    let mut runtime_id = 1;
    let mut nodes: Vec<NodeId> = abstract_flow.get_nodes().map(|(id, _)| *id).collect();
    nodes.sort(); // Ensure consistent ordering

    for node_id in nodes {
        scheduling_config.assign_node(runtime_id, node_id);
        runtime_id = ((runtime_id) % num_runtimes) + 1;
    }

    scheduling_config
}
#[cfg(test)]
mod tests {
    use super::*;
    use flowrs::comm::communication::NodeCommunicator;
    use flowrs::comm::thread_communicator::ThreadCommunicator;
    use flowrs::connection::Edge;
    use flowrs::exec::execution_mode::ExecutionMode;
    use flowrs::exec::execution_node::ExecutionNode;
    use flowrs::node::Node;
    use flowrs::types::type_registry::{POLL_REGISTRY, TYPE_REGISTRY};
    use std::any::TypeId;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_connect_nodes_with_execution_nodes() {
        // 1. Create the flow using the return_dummy_flow function
        let mut abstract_flow = return_dummy_flow().await.expect("Failed to create flow");

        // 2. Extract the nodes from the flow
        let mut nodes = abstract_flow.move_nodes().collect::<HashMap<_, _>>();

        // 3. Initialize the extracted nodes as ExecutionNodes
        let mut execution_nodes: HashMap<u128, Arc<Mutex<ExecutionNode>>> = HashMap::new();

        for (node_id, node) in nodes.drain() {
            let communicator =
                ThreadCommunicator::<String>::new().expect("Failed to create communicator");
            let edge = Edge::new(NodeCommunicator::ThreadComm(communicator));

            let type_ids = HashMap::new(); // Add actual type info if needed
            let execution_node =
                ExecutionNode::new(node, node_id, ExecutionMode::Continuous, edge, type_ids);
            execution_nodes.insert(node_id, Arc::new(Mutex::new(execution_node)));
        }

        // 4. Get the sender and receiver nodes
        let sender_id = 1;
        let receiver_id = 3;
        let sender_node = execution_nodes.get(&sender_id).unwrap().clone();
        let receiver_node = execution_nodes.get(&receiver_id).unwrap().clone();

        // 5. Lock and access IO
        let mut sender_guard = sender_node.lock().await;
        let mut receiver_guard = receiver_node.lock().await;
        let sender_io = sender_guard.get_io_mut();
        let receiver_io = receiver_guard.get_io_mut();

        // 6. Get connection type
        let connection = abstract_flow
            .get_connections()
            .next()
            .expect("No connections found in abstract flow");

        let (_, type_id) = abstract_flow
            .get_connection_type(&connection)
            .expect("Connection type not found");

        // 7. Connect nodes using registered function
        let registry = TYPE_REGISTRY.lock().await;
        let connect_fn = registry
            .get(type_id)
            .expect("Connection function not found in registry.");
        connect_fn(sender_id, receiver_id, 0, 0, sender_io, receiver_io);
        drop(registry);

        tracing::trace!("[test] Successfully connected nodes.");

        // 8. Poll input using POLL_REGISTRY
        let mut poll_registry = POLL_REGISTRY.lock().await;
        let poll_fn = poll_registry
            .get_mut(&TypeId::of::<String>())
            .expect("Polling function for `String` not found");

        // Use the unified erased call for simplicity
        poll_fn.poll(receiver_io, 0).await.expect("Polling failed");

        tracing::debug!("[test] Successfully polled receiver node.");
    }
}
