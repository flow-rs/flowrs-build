use anyhow::Error;
use clap::Parser;
use flowrs::comm::communication::Communicator;
use flowrs::comm::communication::NodeCommunicator;
use flowrs::exec::execution_configuration::ExecutionConfig;
use flowrs::flow::abstract_flow::AbstractFlow;
use flowrs::flow::flow_types::NodeIOIndex;
use flowrs::flow::flow_types::NodeId;
use flowrs::node::ReceiveError;
use flowrs::nodes::node_io::SetupIO;
use flowrs::nodes::node_io::TypedInput;
use flowrs::nodes::node_io::TypedOutput;
use flowrs::sched::scheduling_config::RuntimeId;
use flowrs::sched::scheduling_config::SchedulingConfig;
use flowrs::types::type_registry::PollFn;
use flowrs::types::type_registry::POLL_REGISTRY;
use flowrs::types::type_registry::TYPE_REGISTRY;
use flowrs_build::logging;
use flowrs_build::runtime::node_runtime::NodeRuntime;
use flowrs_build::runtime::orchestrator::Orchestrator;
use flowrs_build::runtime::runtime_args::Arguments;
use std::any::TypeId;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::lookup_host;

#[macro_export]
macro_rules! generate_local_connection {
    ($type:ty) => {
        fn connect_nodes(
            sender_id: NodeId,
            receiver_id: NodeId,
            sender_out_idx: NodeIOIndex,
            recv_in_idx: NodeIOIndex,
            sender_io: &mut dyn SetupIO,
            receiver_io: &mut dyn SetupIO,
        ) {
            tracing::debug!(
                "[DEBUG] Registering connection function for type ID: {:?} (type: {})",
                TypeId::of::<$type>(),
                stringify!($type)
            );

            if let Some(sender_output_any) = sender_io.get_output_communicator(sender_out_idx) {
                if let Some(sender_output_wrapper) = sender_output_any.downcast_mut::<TypedOutput<$type>>() {
                    let sender_output = &mut sender_output_wrapper.output;
                    if let Some(receiver_input_any) = receiver_io.get_input_communicator(recv_in_idx) {
                        if let Some(receiver_input_wrapper) = receiver_input_any.downcast_mut::<TypedInput<$type>>() {
                            let receiver_input = &mut receiver_input_wrapper.input;
                            if let Some(existing_comm) = sender_output.get_communicator_mut() {
                                let send_half = existing_comm.clone_send();
                                let recv_half = existing_comm.move_recv().expect("Failed to move receiver");

                                sender_output.set_communicator(NodeCommunicator::ThreadComm(send_half));
                                receiver_input.set_communicator(NodeCommunicator::ThreadComm(recv_half));

                                tracing::debug!(
                                    "[connect_nodes] Successfully connected nodes {} -> {} with type {}",
                                    sender_id,
                                    receiver_id,
                                    stringify!($type)
                                );
                            } else {
                                panic!("[connect_nodes] No communicator to split!");
                            }
                        } else {
                            panic!("[connect_nodes] Receiver IO type mismatch");
                        }
                    } else {
                        panic!("[connect_nodes] Failed to get input communicator");
                    }
                } else {
                    panic!("[connect_nodes] Sender IO type mismatch");
                }
            } else {
                panic!("[connect_nodes] Failed to get output communicator");
            }
        }

        // Register both local and dynamic factory/setup functions
        let mut registry = TYPE_REGISTRY.lock().await;
        registry.register::<$type>(connect_nodes);                    // local connection
        registry.register_communicator::<$type>(stringify!($type));  // P2P factory + IO setup

         // Register polling function in the separate registry
        let poll_fn: PollFn<$type> = Box::new(|io, idx| {
            Box::pin(async move {
                if let Some(edge_any) = io.get_input_communicator(idx) {
                    let typed_input = edge_any
                        .downcast_mut::<TypedInput<$type>>()
                        .ok_or_else(|| ReceiveError::<$type>::Other(anyhow::anyhow!(
                            "Downcast to TypedInput<{}> failed at index {}",
                            stringify!($type),
                            idx
                        )))?;

                    typed_input.input.edge.poll_and_buffer().await?;
                } else {
                    return Err(ReceiveError::<$type>::Other(anyhow::anyhow!(
                        "No input communicator found at index {}",
                        idx
                    )));
                }

                Ok(())
            })
        });


        let mut poll_registry = POLL_REGISTRY.lock().await;
        poll_registry.register_poll_fn::<$type>(poll_fn);

        tracing::debug!(
            "[generate_local_connection] Fully registered type: {}",
            stringify!($type)
        );
    };
}

#[macro_export]
macro_rules! connect_nodes {
    ($type:ty, $flow:expr, $sender_id:expr, $receiver_id:expr, $sender_out_idx:expr, $recv_in_idx:expr) => {{
        generate_local_connection!($type);
        $flow.connect_nodes::<$type>($sender_id, $receiver_id, $sender_out_idx, $recv_in_idx)
    }};
}

async fn get_orchestrator_address() -> Result<SocketAddr, anyhow::Error> {
    let mut addrs = lookup_host("orchestrator:5000").await?;
    addrs
        .next()
        .ok_or_else(|| anyhow::Error::msg("No valid IP found for orchestrator"))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::init_logging();
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

async fn return_dummy_flow() -> Result<AbstractFlow, Error> {
    use flowrs_std::add::SimpleAddNode;
    use flowrs_std::debug::DebugNode;
    use flowrs_std::value::ValueNode;

    let mut flow = AbstractFlow::new_empty();

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

fn dummy_scheduling(abstract_flow: &AbstractFlow, num_runtimes: u128) -> SchedulingConfig {
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
    use flowrs::node::{ExecutionNode, Node};
    use flowrs::types::type_registry::PollFn;
    use flowrs::types::type_registry::TYPE_REGISTRY;
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
            let execution_node = ExecutionNode::new(node, ExecutionMode::Continuous, edge);
            execution_nodes.insert(node_id, Arc::new(Mutex::new(execution_node)));
        }

        // 4. Get the sender and receiver nodes from the execution nodes
        let sender_id = 1;
        let receiver_id = 3;
        let sender_node = execution_nodes
            .get(&sender_id)
            .expect("Sender node not found")
            .clone();
        let receiver_node = execution_nodes
            .get(&receiver_id)
            .expect("Receiver node not found")
            .clone();

        // 5. Lock the nodes and access their inner nodes
        let mut sender_guard = sender_node.lock().await;
        let mut receiver_guard = receiver_node.lock().await;

        // // Extract the inner nodes from the ExecutionNode
        // let sender_inner_node = &mut sender_guard.node;
        // let receiver_inner_node = &mut receiver_guard.node;

        // Access the IO from the inner node directly
        let sender_io = sender_guard.get_io_mut();
        let receiver_io = receiver_guard.get_io_mut();

        // 6. Get the connection function from the registry using the correct type ID
        let connection = abstract_flow
            .get_connections()
            .next()
            .expect("No connections found in abstract flow");
        let type_id = abstract_flow
            .get_connection_type(&connection)
            .expect("Connection type not found");

        let registry = TYPE_REGISTRY.lock().unwrap();
        let connect_fn = registry.get(type_id).expect(
            "[test_connect_nodes_with_execution_nodes] Connection function not found in registry.",
        );

        // 7. Call the connection function to connect the nodes
        connect_fn(sender_id, receiver_id, 0, 0, sender_io, receiver_io);

        tracing::debug!("[test_connect_nodes_with_execution_nodes] Successfully connected nodes.");

        //8. Test polling function
        // Unlock registry again (separate scope since it's already locked above)
        drop(registry); // drop first lock

        let mut registry = TYPE_REGISTRY.lock().unwrap();

        let poll_fn = registry
            .get_poll_fn::<String>()
            .expect("Polling function for `String` not found")
            .downcast_mut::<PollFn<String>>()
            .expect("Failed to downcast polling function");

        // Call the poll function on the receiver (inputs are always polled)
        poll_fn(receiver_io).await.expect("Polling failed");

        tracing::debug!(
            "[test_connect_nodes_with_execution_nodes] Successfully polled receiver node."
        );
    }
}
