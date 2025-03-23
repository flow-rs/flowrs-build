use crate::runtime;
use crate::runtime::runtime_constants::{RUNTIME_PORT, SETUP_PORT};
use anyhow::Error;
use flowrs::comm::communication::Communicator;
use flowrs::comm::communication::NodeCommunicator;
use flowrs::comm::messages::Message;
use flowrs::comm::network_communicator::NetworkCommunicator;
use flowrs::comm::thread_communicator::ThreadCommunicator;
use flowrs::exec::execution::StandardExecutor;
use flowrs::exec::execution_configuration::ExecutionConfig;
use flowrs::flow::abstract_flow::{self, AbstractFlow};
use flowrs::flow::flow_connection::FlowNodeConnection;
use flowrs::flow::flow_types::{NodeIOIndex, NodeId};
use flowrs::node::{ExecutionNode, Node};
use flowrs::nodes::node_io::{
    SettableCommunicator, SetupInputCommunicator, SetupOutputCommunicator, SplittableCommunicator,
    TypedInput, TypedOutput,
};
use flowrs::types::type_registry::TYPE_REGISTRY;
use flowrs_package::flow_package::package::Type;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{sleep, timeout};

use super::runtime_args::Arguments;

pub struct NodeRuntime {
    abstract_flow: AbstractFlow,
    orchestrator_ip: String,
    runtime_id: u128,
    assigned_port: Option<u16>,
    execution_config: ExecutionConfig,
    executor: Arc<Mutex<StandardExecutor>>,
    sender: Arc<Mutex<NetworkCommunicator>>,
    receiver: Arc<Mutex<NetworkCommunicator>>,
    node_id_map: HashMap<NodeId, String>,
}

impl NodeRuntime {
    pub async fn new(
        orchestrator_ip: String,
        runtime_id: u128,
        abstract_flow: AbstractFlow,
        execution_config: ExecutionConfig,
    ) -> Result<Self, Error> {
        let sender = Arc::new(Mutex::new(
            NetworkCommunicator::new().await.expect("should construct!"),
        ));
        let receiver = Arc::new(Mutex::new(
            NetworkCommunicator::new().await.expect("should construct!"),
        ));
        let executor = Arc::new(Mutex::new(StandardExecutor::new()));

        Ok(Self {
            abstract_flow,
            orchestrator_ip,
            runtime_id,
            assigned_port: None,
            execution_config,
            executor,
            sender,
            receiver,
            node_id_map: HashMap::new(),
        })
    }

    pub async fn run(&mut self, args: Arguments, orch_addr: SocketAddr) -> Result<(), Error> {
        println!(
            "[Node RT] Running as node runtime with flow file: {}",
            args.flow
        );

        let orchestrator_ip = orch_addr.ip().to_string();
        let mut assigned_port: u16 = 0;
        let runtime_id = self.execution_config.runtime_id;

        // Shared oneshot channel to signal receiver readiness
        let (tx, rx) = tokio::sync::oneshot::channel();

        // Shared receiver reference to persist across retries
        let receiver_shared = Arc::clone(&self.receiver);
        let orchestrator_ip_clone = orchestrator_ip.clone();

        // =======================================================================================
        // Step 1: Create a receiver communicator to receive on RUNTIME_PORT
        // =======================================================================================
        let _recv_task = task::spawn(async move {
            // Signal readiness as soon as the receiver object is created
            // {
            //     let mut receiver_guard = receiver_shared.lock().await;
            //     //*receiver_guard = Some(orch_receiver);
            //     let _ = tx.send(());
            // }

            // Now perform the blocking call using the stored reference
            let mut receiver_guard = receiver_shared.lock().await;
            let _ = tx.send(());
            //if let Some(ref mut receiver) = *receiver_guard {
            match <NetworkCommunicator as Communicator<String>>::connect_recv::<'_, '_>(
                &mut *receiver_guard,
                Some(orchestrator_ip_clone),
                Some(RUNTIME_PORT),
            )
            .await
            {
                Ok(_) => {
                    println!(
                        "[Node RT] Receiver successfully established on port {}",
                        RUNTIME_PORT
                    );
                }
                Err(e) => {
                    println!(
                        "[Node RT] Receiver failed to bind or accept connection: {}",
                        e
                    );
                }
            }
        });
        // =======================================================================================
        // Step 2: Connect to orchestrator on SETUP_PORT while receiver is listening
        // =======================================================================================
        let mut retries = 0;
        let max_retries = 10;

        // looping for retries if a timeout occurs
        'receive_loop: while retries < max_retries {
            retries += 1;
            'send_loop: while retries < max_retries {
                match TcpStream::connect((orchestrator_ip.clone(), SETUP_PORT)).await {
                    Ok(mut stream) => {
                        println!(
                            "[Node RT] Successfully signaled presence to orchestrator at {}:{}",
                            orchestrator_ip, SETUP_PORT
                        );
                        // Serialize and send the runtime ID
                        let id_msg = format!("RUNTIME_ID:{}", runtime_id);
                        if let Err(e) = stream.write_all(id_msg.as_bytes()).await {
                            println!(
                                "[Node RT] ERROR: Failed to send Runtime ID to Orchestrator! {}",
                                e
                            );
                            return Err(Error::msg("Failed to send Runtime ID"));
                        }

                        println!("[Node RT] Successfully sent Runtime ID to Orchestrator");
                        break 'send_loop;
                    }
                    Err(_) => {
                        println!(
                    "[Node RT] Failed to connect to orchestrator setup port. Retrying... ({}/{})",
                    retries,
                    max_retries
                );
                        sleep(Duration::from_secs(1)).await;
                        retries += 1;
                    }
                }
            }

            if retries == max_retries {
                println!("[Node RT] Could not signal presence to orchestrator. Exiting.");
                return Err(anyhow::Error::msg(
                    "Failed to establish setup connection with orchestrator",
                ));
            }

            // =======================================================================================
            // Step 3: Wait for Receiver Task and Receive Assigned Port
            // =======================================================================================
            sleep(Duration::from_millis(100)).await;
            let recv_timeout = Duration::from_secs(2);

            // Wait for the receiver task to complete and retrieve the communicator
            println!("[Node RT] Waiting for assigned communication port from orchestrator...");

            let receiver_shared = Arc::clone(&self.receiver);
            let mut receiver_guard = receiver_shared.lock().await;
            let receiver = &mut *receiver_guard;
            match timeout(recv_timeout, async {
                loop {
                    //if let Some(receiver) = &mut *receiver_guard {
                    return Communicator::<String>::receive(receiver).await;
                    //}
                    sleep(Duration::from_millis(500)).await; // Wait before checking again
                }
            })
            .await
            {
                Ok(Ok(Message::SetupCommunicationPort(port))) => {
                    assigned_port = port;
                    println!("[Node RT] Received assigned port: {}", assigned_port);
                    break 'receive_loop;
                }
                Ok(Ok(_)) => {
                    println!("[Node RT] Unexpected message received. Retrying...");
                }
                Ok(Err(e)) => {
                    println!("[Node RT] Error receiving message: {}. Retrying...", e);
                }
                Err(_) => {
                    println!("[Node RT] Timed out waiting for message. Retrying...");
                }
            }

            if retries >= max_retries {
                return Err(anyhow::Error::msg(
            "[Node RT] [FATAL] Failed to establish connection with orchestrator after multiple attempts.",
        ));
            }
            sleep(Duration::from_secs(3)).await;
        }
        // let mut orch_receiver = rx.await.expect("should receive");
        // let assigned_port_msg: Message<String> =
        //     orch_receiver.receive().await.expect("should receive");

        // if let Message::SetupCommunicationPort(port) = assigned_port_msg {
        //     assigned_port = port;
        //     println!(
        //         "[Node RT] Received assigned port from orchestrator: {}",
        //         assigned_port
        //     );
        // } else {
        //     return Err(anyhow::Error::msg(
        //         "[Node RT] Unexpected message received instead of assigned port!",
        //     ));
        // }

        // =======================================================================================
        // Step 4: Create a sender to orchestrator on the assigned port
        // =======================================================================================
        let mut orch_sender = NetworkCommunicator::new().await.expect("should construct");
        match <NetworkCommunicator as Communicator<String>>::connect_send::<'_, '_>(
            &mut orch_sender,
            Some(orchestrator_ip.clone()),
            Some(assigned_port),
        )
        .await
        {
            Ok(_) => {
                println!(
                    "[Node RT] Sender established. Sending messages to orchestrator at {}:{}",
                    orchestrator_ip, assigned_port
                );
            }
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "[Node RT] Failed to establish sender: {}",
                    e
                )));
            }
        }
        // =======================================================================================
        // Step 5: Send an acknowledging message back to the orchestrator
        // =======================================================================================
        let ack_message = Message::<String>::AcknowledgeConnection;

        println!(
            "[Node RT] Sending acknowledgment message to orchestrator on port {}...",
            assigned_port
        );

        if let Err(e) = orch_sender.send(ack_message).await {
            println!(
                "[Node RT] Failed to send acknowledgment message to orchestrator: {}",
                e
            );
            return Err(anyhow::Error::msg(
                "Failed to send acknowledgment message to orchestrator",
            ));
        }

        println!(
            "[Node RT] Successfully connected to orchestrator on {}:{} (send) and {}:{} (receive)",
            orchestrator_ip, assigned_port, orchestrator_ip, RUNTIME_PORT
        );

        // =======================================================================================
        // Step 6: Node Initialization
        // =======================================================================================
        // Await the readiness signal before proceeding
        if rx.await.is_err() {
            return Err(anyhow::Error::msg("Receiver setup failed"));
        }

        // MAIN RECEIVE LOOP
        let receiver_shared = Arc::clone(&self.receiver);
        loop {
            let mut receiver_guard = receiver_shared.lock().await;
            let receiver = &mut *receiver_guard;
            //if let Some(receiver) = &mut *receiver_guard {
            println!(
                "[Node RT] Waiting for InitializeLocalNodes on port {}",
                assigned_port
            );
            match Communicator::<String>::receive(receiver).await {
                Ok(Message::InitializeLocalNodes) => {
                    println!("[Node RT] Received InitializeLocalNodes request...");
                    let executor = Arc::clone(&self.executor);
                    let mut executor_guard = executor.lock().await;
                    // Perform actual node initialization
                    if let Err(e) = executor_guard
                        .initialize_nodes(&mut self.abstract_flow, &self.execution_config)
                        .await
                    {
                        println!("[Node RT] ERROR: Failed to initialize nodes: {}", e);
                        return Err(anyhow::Error::msg("Node initialization failed"));
                    }

                    println!("[Node RT] Successfully initialized local nodes.");

                    // Call `on_ready()` for all nodes after initialization
                    for node in executor_guard.execution_nodes.values() {
                        let mut node_guard = node.lock().await;
                        if let Err(e) = node_guard.on_ready() {
                            println!("[Node RT] ERROR: Node failed to enter ready state: {}", e);
                            return Err(anyhow::Error::msg("Node ready state failed"));
                        }
                    }
                    drop(executor_guard);

                    println!("[Node RT] All nodes are ready!");

                    // Send acknowledgment back to the orchestrator
                    let ack_message = Message::<String>::AcknowledgeNodeInitialization;
                    if let Err(e) = orch_sender.send(ack_message).await {
                        println!(
                            "[Node RT] ERROR: Failed to send AcknowledgeNodeInitialization: {}",
                            e
                        );
                    } else {
                        println!("[Node RT] Sent AcknowledgeNodeInitialization.");
                    }
                }
                Ok(Message::OrchestratorRequestNodeConnection(
                    sender_id,
                    receiver_id,
                    runtime_id,
                    runtime_ip,
                    sender_out_idx,
                    recv_in_idx,
                )) => {
                    println!(
                        "[Node RT] Received P2P connection request: {} -> {} (Out {} -> In {})",
                        sender_id, receiver_id, sender_out_idx, recv_in_idx
                    );

                    let connection = FlowNodeConnection {
                        sender_id,
                        receiver_id,
                        send_out_idx: sender_out_idx, // Updated to u128
                        recv_in_idx,                  // Updated to u128
                    };

                    let connection_type = self.abstract_flow.get_connection_type(&connection);
                    if connection_type.is_none() {
                        println!(
                                "[Node RT] WARNING: P2P request does not match any known connection: {} -> {}",
                                sender_id, receiver_id
                            );
                        return Err(anyhow::Error::msg("Invalid P2P connection request"));
                    }

                    //let connection_type = connection_type.unwrap();

                    // Determine if connection is local or remote
                    //let is_local =
                    &self
                        .execution_config
                        .is_local_connection(sender_id, receiver_id);

                    //let runtime = Arc::new(Mutex::new(self));
                    if let Err(e) = self
                        .handle_p2p_connection(sender_id, receiver_id, sender_out_idx, recv_in_idx)
                        .await
                    {
                        println!("[Node RT] ERROR: Failed to setup P2P connection: {}", e);
                    }
                }
                Ok(Message::StartExecution) => {
                    println!("[Node RT] Received StartExecution command. Beginning execution...");

                    if let Err(e) = self.start_execution().await {
                        println!("[Node RT] ERROR: Execution failed: {}", e);
                    } else {
                        println!("[Node RT] Execution completed successfully.");
                    }
                }
                Ok(Message::RequestNodeRuntimeIP(node_id)) => {
                    println!(
                        "[Node RT] Received request for Node {}'s IP from Orchestrator...",
                        node_id
                    );

                    // Check if we already have the IP stored
                    if let Some(ip) = self.node_id_map.get(&node_id) {
                        println!("[Node RT] Sending cached IP for Node {}: {}", node_id, ip);
                        let response = Message::<String>::RespondNodeRuntimeIP(node_id, ip.clone());
                        let mut sender_guard = self.sender.lock().await;
                        if let Err(e) = sender_guard.send(response).await {
                            println!("[Node RT] ERROR: Failed to send IP response: {}", e);
                        }
                    } else {
                        println!("[Node RT] WARNING: No known IP for Node {}!", node_id);
                    }
                }

                Ok(Message::RespondNodeRuntimeIP(requested_node_id, ip)) => {
                    println!(
                        "[Node RT] Received IP for Node {}: {}",
                        requested_node_id, ip
                    );
                    self.node_id_map.insert(requested_node_id, ip);
                }
                Ok(msg) => {
                    println!(
                        "[Node RT] WARNING: Unexpected message from orchestrator: {:?}",
                        msg
                    );
                }
                Err(e) => {
                    println!(
                        "[Node RT] ERROR: Failed to receive message from orchestrator: {}",
                        e
                    );
                }
            }
        }

        // Keep the runtime running indefinitely
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }

        Ok(())
    }

    /// Handles the incoming P2P connection request from the orchestrator
    async fn handle_p2p_connection(
        &mut self,
        sender_id: NodeId,
        receiver_id: NodeId,
        sender_out_idx: u128,
        recv_in_idx: u128,
    ) -> Result<(), Error> {
        println!(
            "[Node RT] Handling P2P connection request: {} -> {} (Out {} -> In {})",
            sender_id, receiver_id, sender_out_idx, recv_in_idx
        );

        // Step 1: Ensure the current runtime owns the sender node
        let executor = Arc::clone(&self.executor);
        let is_local = self
            .execution_config
            .is_local_connection(sender_id, receiver_id);

        if is_local {
            println!(
                "[Node RT] Establishing local connection between nodes {} and {}...",
                sender_id, receiver_id
            );

            // Step 3A-1: Retrieve the types of the sender and receiver nodes
            let out_type_id = self
                .abstract_flow
                .get_output_type(sender_id, sender_out_idx)
                .ok_or_else(|| {
                    anyhow::Error::msg(format!(
                        "Type not found: sender out index {}",
                        sender_out_idx
                    ))
                })?;

            let in_type_id = self
                .abstract_flow
                .get_input_type(receiver_id, recv_in_idx)
                .ok_or_else(|| {
                    anyhow::Error::msg(format!("Type not found: receiver in index {}", recv_in_idx))
                })?;

            if out_type_id != in_type_id {
                return Err(anyhow::Error::msg(format!(
                    "Type mismatch: sender out index {} => {:?}, receiver in index {} => {:?}",
                    sender_out_idx, out_type_id, recv_in_idx, in_type_id
                )));
            }

            // Step 3A-2: Clone the sender and receiver node Arcs
            let (sender_exec_node, receiver_exec_node) = {
                let executor_guard = executor.lock().await;

                let sender_node = executor_guard
                    .execution_nodes
                    .get(&sender_id)
                    .cloned()
                    .ok_or_else(|| {
                        anyhow::Error::msg(format!("Sender node {} not found", sender_id))
                    })?;

                let receiver_node = executor_guard
                    .execution_nodes
                    .get(&receiver_id)
                    .cloned()
                    .ok_or_else(|| {
                        anyhow::Error::msg(format!("Receiver node {} not found", receiver_id))
                    })?;

                (sender_node, receiver_node)
            };

            // Step 3A-3: Lock the nodes and call the dynamic connection function directly
            {
                let mut sender_guard = sender_exec_node.lock().await;
                let mut receiver_guard = receiver_exec_node.lock().await;

                let sender_io = sender_guard.node.get_io_mut().as_any_mut();
                let receiver_io = receiver_guard.node.get_io_mut().as_any_mut();

                // Call the dynamic connection function directly
                Self::connect_nodes_dynamic(
                    out_type_id,
                    sender_id,
                    receiver_id,
                    sender_out_idx,
                    recv_in_idx,
                    sender_io,
                    receiver_io,
                );
            }

            println!(
                "[Node RT] Successfully connected local nodes {} -> {}",
                sender_id, receiver_id
            );
        } else {
            println!(
                "[Node RT] Establishing remote connection between nodes {} and {}...",
                sender_id, receiver_id
            );
            // Remote connection handling would go here...
        }

        // Step 4: Send acknowledgment back to the orchestrator
        let ack_message = Message::<String>::AcknowledgeConnectionSetup(
            sender_id,
            receiver_id,
            recv_in_idx.try_into().expect("Index too large for u16"),
        );

        if let Err(e) = self.sender.lock().await.send(ack_message).await {
            println!(
                "[Node RT] ERROR: Failed to send AcknowledgeConnectionSetup: {}",
                e
            );
            return Err(anyhow::Error::msg(
                "Failed to send AcknowledgeConnectionSetup",
            ));
        }

        println!(
            "[Node RT] Sent AcknowledgeConnectionSetup for P2P connection {} -> {}.",
            sender_id, receiver_id
        );

        Ok(())
    }
    // async fn connect_nodes_sync(
    //     &self,
    //     sender_node: Arc<Mutex<ExecutionNode>>,
    //     receiver_node: Arc<Mutex<ExecutionNode>>,
    //     sender_id: NodeId,
    //     receiver_id: NodeId,
    //     sender_out_idx: NodeIOIndex,
    //     recv_in_idx: NodeIOIndex,
    //     out_type_id: TypeId,
    // ) -> Result<(), anyhow::Error> {
    //     // Lock each node and access their IO in an async context
    //     let mut sender_guard = sender_node.lock().await;

    //     let mut receiver_guard = receiver_node.lock().await;

    //     let sender_io = sender_guard.node.get_io_mut().as_any_mut();
    //     let receiver_io = receiver_guard.node.get_io_mut().as_any_mut();

    //     println!("[DEBUG] Actual sender_io type: {:?}", sender_io.type_id());
    //     println!(
    //         "[DEBUG] Expected sender_io type: {:?}",
    //         std::any::TypeId::of::<TypedOutput<u32>>()
    //     );

    //     // Call the dynamic connection function directly
    //     Self::connect_nodes_dynamic(
    //         out_type_id,
    //         sender_id,
    //         receiver_id,
    //         sender_out_idx,
    //         recv_in_idx,
    //         sender_io,
    //         receiver_io,
    //     );

    //     println!(
    //         "[Node RT] Successfully connected local nodes {} -> {}",
    //         sender_id, receiver_id
    //     );

    //     Ok(())
    // }

    fn connect_nodes_dynamic(
        type_id: TypeId,
        sender_id: NodeId,
        receiver_id: NodeId,
        sender_out_idx: NodeIOIndex,
        recv_in_idx: NodeIOIndex,
        sender_io: &mut dyn Any,
        receiver_io: &mut dyn Any,
    ) {
        if let Some(connect_fn) = TYPE_REGISTRY.lock().unwrap().get(type_id) {
            // println!(
            //     "[Node RT] Found registered function for type ID: {:?}",
            //     type_id
            // );

            // // Check the actual type of sender_io by casting to Any and obtaining the type ID
            // let actual_sender_type_id = (*sender_io).type_id();
            // println!(
            //     "[Node RT] Actual sender IO type ID: {:?}",
            //     actual_sender_type_id
            // );

            // if actual_sender_type_id != type_id {
            //     println!(
            //         "[Node RT] ERROR: Sender output type mismatch. Expected {:?}, but got {:?}",
            //         type_id, actual_sender_type_id
            //     );
            //     panic!("Sender output type mismatch");
            // }

            println!(
                "[DEBUG] Registering connection for type ID: {:?}",
                TypeId::of::<u32>()
            );
            println!("[DEBUG] Retrieved type ID for connection: {:?}", type_id);

            connect_fn(
                sender_id,
                receiver_id,
                sender_out_idx,
                recv_in_idx,
                sender_io,
                receiver_io,
            );
            println!("[Node RT] Successfully connected nodes using the dynamic function lookup.");
        } else {
            panic!(
                "[Node RT] No connection function found for type ID {:?}",
                type_id
            );
        }
    }

    async fn start_execution(&self) -> Result<(), anyhow::Error> {
        println!("[Node RT] Starting execution of local nodes...");
        let executor = Arc::clone(&self.executor);
        let executor_guard = executor.lock().await;
        // Make sure nodes are ready
        if let Err(e) = executor_guard.ready_nodes().await {
            println!("[Node RT] ERROR: Node readiness failed: {}", e);
            return Err(e);
        }

        // Start execution through executor
        executor_guard.start_execution().await;
        println!("[Node RT] Execution started successfully.");
        Ok(())
    }
}
