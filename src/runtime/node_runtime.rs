use crate::runtime::runtime_constants::{R2R_PORT, RUNTIME_PORT, SETUP_PORT};
use anyhow::anyhow;
use anyhow::Error;
use flowrs::comm::communication::Communicator;
use flowrs::comm::messages::Message;
use flowrs::comm::network_communicator::NetworkCommunicator;
use flowrs::exec::execution::StandardExecutor;
use flowrs::exec::execution_configuration::{ExecutionConfig, NodeConfig};
use flowrs::flow::abstract_flow::AbstractFlow;
use flowrs::flow::flow_connection::FlowNodeConnection;
use flowrs::flow::flow_types::{NodeIOIndex, NodeId};
use flowrs::node::Node;
use flowrs::nodes::node_io::SetupIO;
use flowrs::types::type_registry::TYPE_REGISTRY;
use std::any::TypeId;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};

use super::runtime_args::Arguments;

pub struct NodeRuntime {
    abstract_flow: Arc<Mutex<AbstractFlow>>,
    orchestrator_ip: String,
    _runtime_id: u128,
    assigned_port: Option<u16>,
    execution_config: ExecutionConfig,
    executor: Arc<Mutex<StandardExecutor>>,
    orch_sender: Arc<Mutex<NetworkCommunicator<String>>>,
    orch_receiver: Arc<Mutex<NetworkCommunicator<String>>>,
    _r2r_receiver: Arc<Mutex<NetworkCommunicator<String>>>,
    node_id_map: Arc<Mutex<HashMap<NodeId, String>>>,
}

impl NodeRuntime {
    pub async fn new(
        orchestrator_ip: String,
        _runtime_id: u128,
        abstract_flow: AbstractFlow,
        execution_config: ExecutionConfig,
    ) -> Result<Self, Error> {
        let orch_sender = Arc::new(Mutex::new(
            NetworkCommunicator::new().await.expect("should construct!"),
        ));
        let orch_receiver = Arc::new(Mutex::new(
            NetworkCommunicator::new().await.expect("should construct!"),
        ));
        let _r2r_receiver = Arc::new(Mutex::new(
            NetworkCommunicator::new().await.expect("should construct!"),
        ));
        let executor = Arc::new(Mutex::new(StandardExecutor::new()));
        let node_id_map = Arc::new(Mutex::new(HashMap::<NodeId, String>::new()));
        let abstract_flow = Arc::new(Mutex::new(abstract_flow));

        Ok(Self {
            abstract_flow,
            orchestrator_ip,
            _runtime_id,
            assigned_port: None,
            execution_config,
            executor,
            orch_sender,
            orch_receiver,
            _r2r_receiver,
            node_id_map,
        })
    }

    // pub async fn run(&mut self, args: Arguments, orch_addr: SocketAddr) -> Result<(), Error> {
    //     println!(
    //         "[Node RT] Running as node runtime with flow file: {}",
    //         args.flow
    //     );

    //     let orchestrator_ip = orch_addr.ip().to_string();
    //     let mut assigned_port: u16 = 0;
    //     let runtime_id = self.execution_config.runtime_id;

    //     // Shared oneshot channel to signal receiver readiness
    //     let (tx, rx) = tokio::sync::oneshot::channel();

    //     // Shared receiver reference to persist across retries
    //     let orch_receiver_shared: Arc<Mutex<NetworkCommunicator<String>>> =
    //         Arc::clone(&self.orch_receiver);
    //     let r2r_receiver_shared: Arc<Mutex<NetworkCommunicator<String>>> =
    //         Arc::clone(&self.r2r_receiver);
    //     let orchestrator_ip_clone = orchestrator_ip.clone();

    //     // =======================================================================================
    //     // Step 1: Create a receiver communicator to receive on RUNTIME_PORT
    //     // =======================================================================================
    //     let _orch_recv_task = task::spawn(async move {
    //         // Now perform the blocking call using the stored reference
    //         let mut receiver_guard = orch_receiver_shared.lock().await;
    //         let _ = tx.send(());
    //         //if let Some(ref mut receiver) = *receiver_guard {
    //         match &mut receiver_guard
    //             .connect_recv(Some(orchestrator_ip_clone), Some(RUNTIME_PORT))
    //             .await
    //         {
    //             Ok(_) => {
    //                 println!(
    //                     "[Node RT] Receiver successfully established on port {}",
    //                     RUNTIME_PORT
    //                 );
    //             }
    //             Err(e) => {
    //                 println!(
    //                     "[Node RT] Receiver failed to bind or accept connection: {}",
    //                     e
    //                 );
    //             }
    //         }
    //     });
    //     let _r2r_recv_task = task::spawn(async move {
    //         // Now perform the blocking call using the stored reference
    //         let mut receiver_guard = r2r_receiver_shared.lock().await;
    //         //let _ = tx.send(());
    //         //if let Some(ref mut receiver) = *receiver_guard {
    //         match &mut receiver_guard
    //             // we use the orchestrator ip here, beacuse the connect_recv() expects an ip, but other ips can still connect.
    //             // TODO
    //             .connect_recv(Some(orchestrator_ip_clone), Some(R2R_PORT))
    //             .await
    //         {
    //             Ok(_) => {
    //                 println!(
    //                     "[Node RT] Receiver successfully established on port {}",
    //                     R2R_PORT
    //                 );
    //             }
    //             Err(e) => {
    //                 println!(
    //                     "[Node RT] Receiver failed to bind or accept connection: {}",
    //                     e
    //                 );
    //             }
    //         }
    //     });

    //     let r2r_handler = task::spawn(async move {
    //         loop {
    //             let mut r2r_guard = r2r_receiver_shared.lock().await;
    //             let r2r_recv = &mut *r2r_guard;

    //             match Communicator::<String>::receive(r2r_recv).await {
    //                 Ok(Message::RequestPeerConnection(
    //                     sending_node_id,
    //                     receiving_node_id,
    //                     sender_out_idx,
    //                     receiver_in_idx,
    //                     type_name,
    //                 )) => {
    //                     self.handle_request_peer_connection(
    //                         sending_node_id,
    //                         receiving_node_id,
    //                         sender_out_idx,
    //                         receiver_in_idx,
    //                         type_name,
    //                     )
    //                     .await;
    //                 }
    //                 Ok(msg) => {
    //                     println!("[Node RT] [R2R] Unhandled message: {:?}", msg);
    //                 }
    //                 Err(e) => {
    //                     println!("[Node RT] [R2R] Error receiving message: {}", e);
    //                 }
    //             }
    //         }
    //     });

    //     // =======================================================================================
    //     // Step 2: Connect to orchestrator on SETUP_PORT while receiver is listening
    //     // =======================================================================================
    //     let mut retries = 0;
    //     let max_retries = 10;

    //     // looping for retries if a timeout occurs
    //     'receive_loop: while retries < max_retries {
    //         retries += 1;
    //         'send_loop: while retries < max_retries {
    //             match TcpStream::connect((orchestrator_ip.clone(), SETUP_PORT)).await {
    //                 Ok(mut stream) => {
    //                     println!(
    //                         "[Node RT] Successfully signaled presence to orchestrator at {}:{}",
    //                         orchestrator_ip, SETUP_PORT
    //                     );
    //                     // Serialize and send the runtime ID
    //                     let id_msg = format!("RUNTIME_ID:{}", runtime_id);
    //                     if let Err(e) = stream.write_all(id_msg.as_bytes()).await {
    //                         println!(
    //                             "[Node RT] ERROR: Failed to send Runtime ID to Orchestrator! {}",
    //                             e
    //                         );
    //                         return Err(Error::msg("Failed to send Runtime ID"));
    //                     }

    //                     println!("[Node RT] Successfully sent Runtime ID to Orchestrator");
    //                     break 'send_loop;
    //                 }
    //                 Err(_) => {
    //                     println!(
    //                 "[Node RT] Failed to connect to orchestrator setup port. Retrying... ({}/{})",
    //                 retries,
    //                 max_retries
    //             );
    //                     sleep(Duration::from_secs(1)).await;
    //                     retries += 1;
    //                 }
    //             }
    //         }

    //         if retries == max_retries {
    //             println!("[Node RT] Could not signal presence to orchestrator. Exiting.");
    //             return Err(anyhow::Error::msg(
    //                 "Failed to establish setup connection with orchestrator",
    //             ));
    //         }

    //         // =======================================================================================
    //         // Step 3: Wait for Receiver Task and Receive Assigned Port
    //         // =======================================================================================
    //         sleep(Duration::from_millis(100)).await;
    //         let recv_timeout = Duration::from_secs(2);

    //         // Wait for the receiver task to complete and retrieve the communicator
    //         println!("[Node RT] Waiting for assigned communication port from orchestrator...");

    //         let receiver_shared: Arc<Mutex<NetworkCommunicator<String>>> =
    //             Arc::clone(&self.orch_receiver);
    //         let mut receiver_guard = receiver_shared.lock().await;
    //         let receiver = &mut *receiver_guard;
    //         match timeout(recv_timeout, async {
    //             loop {
    //                 //if let Some(receiver) = &mut *receiver_guard {
    //                 return Communicator::<String>::receive(receiver).await;
    //                 //}
    //                 sleep(Duration::from_millis(500)).await; // Wait before checking again
    //             }
    //         })
    //         .await
    //         {
    //             Ok(Ok(Message::SetupCommunicationPort(port))) => {
    //                 assigned_port = port;
    //                 println!("[Node RT] Received assigned port: {}", assigned_port);
    //                 break 'receive_loop;
    //             }
    //             Ok(Ok(_)) => {
    //                 println!("[Node RT] Unexpected message received. Retrying...");
    //             }
    //             Ok(Err(e)) => {
    //                 println!("[Node RT] Error receiving message: {}. Retrying...", e);
    //             }
    //             Err(_) => {
    //                 println!("[Node RT] Timed out waiting for message. Retrying...");
    //             }
    //         }

    //         if retries >= max_retries {
    //             return Err(anyhow::Error::msg(
    //         "[Node RT] [FATAL] Failed to establish connection with orchestrator after multiple attempts.",
    //     ));
    //         }
    //         sleep(Duration::from_secs(3)).await;
    //     }
    //     // let mut orch_receiver = rx.await.expect("should receive");
    //     // let assigned_port_msg: Message<String> =
    //     //     orch_receiver.receive().await.expect("should receive");

    //     // if let Message::SetupCommunicationPort(port) = assigned_port_msg {
    //     //     assigned_port = port;
    //     //     println!(
    //     //         "[Node RT] Received assigned port from orchestrator: {}",
    //     //         assigned_port
    //     //     );
    //     // } else {
    //     //     return Err(anyhow::Error::msg(
    //     //         "[Node RT] Unexpected message received instead of assigned port!",
    //     //     ));
    //     // }

    //     // =======================================================================================
    //     // Step 4: Create a sender to orchestrator on the assigned port
    //     // =======================================================================================
    //     {
    //         let mut sender_guard = self.orch_sender.lock().await;
    //         *sender_guard = NetworkCommunicator::new().await.expect("should construct");
    //         match &mut sender_guard
    //             .connect_send(Some(orchestrator_ip.clone()), Some(assigned_port))
    //             .await
    //         {
    //             Ok(_) => {
    //                 println!(
    //                     "[Node RT] Sender established. Sending messages to orchestrator at {}:{}",
    //                     orchestrator_ip, assigned_port
    //                 );
    //             }
    //             Err(e) => {
    //                 return Err(anyhow::Error::msg(format!(
    //                     "[Node RT] Failed to establish sender: {}",
    //                     e
    //                 )));
    //             }
    //         }
    //     }
    //     // =======================================================================================
    //     // Step 5: Send an acknowledging message back to the orchestrator
    //     // =======================================================================================
    //     let ack_message = Message::<String>::AcknowledgeConnection;

    //     println!(
    //         "[Node RT] Sending acknowledgment message to orchestrator on port {}...",
    //         assigned_port
    //     );
    //     {
    //         let mut sender_guard = self.orch_sender.lock().await;
    //         if let Err(e) = sender_guard.send(ack_message).await {
    //             println!(
    //                 "[Node RT] Failed to send acknowledgment message to orchestrator: {}",
    //                 e
    //             );
    //             return Err(anyhow::Error::msg(
    //                 "Failed to send acknowledgment message to orchestrator",
    //             ));
    //         }
    //     }

    //     println!(
    //         "[Node RT] Successfully connected to orchestrator on {}:{} (send) and {}:{} (receive)",
    //         orchestrator_ip, assigned_port, orchestrator_ip, RUNTIME_PORT
    //     );

    //     // =======================================================================================
    //     // Step 6: Node Initialization
    //     // =======================================================================================
    //     // Await the readiness signal before proceeding
    //     if rx.await.is_err() {
    //         return Err(anyhow::Error::msg("Receiver setup failed"));
    //     }

    //     // MAIN RECEIVE LOOP
    //     let receiver_shared: Arc<Mutex<NetworkCommunicator<String>>> =
    //         Arc::clone(&self.orch_receiver);
    //     loop {
    //         let mut receiver_guard = receiver_shared.lock().await;
    //         let receiver = &mut *receiver_guard;
    //         //if let Some(receiver) = &mut *receiver_guard {
    //         println!(
    //             "[Node RT] Waiting for InitializeLocalNodes on port {}",
    //             assigned_port
    //         );
    //         match Communicator::<String>::receive(receiver).await {
    //             Ok(Message::InitializeLocalNodes) => self.handle_initialize_local_nodes().await?,
    //             Ok(Message::OrchestratorRequestNodeConnection(
    //                 sender_id,
    //                 receiver_id,
    //                 runtime_id,
    //                 runtime_ip,
    //                 sender_out_idx,
    //                 recv_in_idx,
    //             )) => {
    //                 self.handle_orchestrator_request_connection(
    //                     sender_id,
    //                     receiver_id,
    //                     runtime_id,
    //                     runtime_ip,
    //                     sender_out_idx,
    //                     recv_in_idx,
    //                 )
    //                 .await?
    //             }
    //             Ok(Message::StartExecution) => self.handle_start_execution().await?,
    //             Ok(Message::RequestNodeRuntimeIP(node_id)) => {
    //                 self.handle_request_node_runtime_ip(node_id).await
    //             }
    //             Ok(Message::RespondNodeRuntimeIP(requested_node_id, ip)) => {
    //                 self.handle_respond_node_runtime_ip(requested_node_id, ip)
    //             }
    //             Ok(Message::RequestPeerConnection(
    //                 sending_node_id,
    //                 receiving_node_id,
    //                 sender_out_idx,
    //                 receiver_in_idx,
    //                 type_name,
    //             )) => {
    //                 self.handle_request_peer_connection(
    //                     sending_node_id,
    //                     receiving_node_id,
    //                     sender_out_idx,
    //                     receiver_in_idx,
    //                     type_name,
    //                 )
    //                 .await?
    //             }
    //             Ok(Message::AcceptPeerConnection(
    //                 sending_node_id,
    //                 receiving_node_id,
    //                 sender_out_idx,
    //                 receiver_in_idx,
    //                 recv_port,
    //             )) => {
    //                 self.handle_accept_peer_connection(
    //                     sending_node_id,
    //                     receiving_node_id,
    //                     sender_out_idx,
    //                     receiver_in_idx,
    //                     recv_port,
    //                 )
    //                 .await?
    //             }
    //             Ok(msg) => {
    //                 println!(
    //                     "[Node RT] WARNING: Unexpected message from orchestrator: {:?}",
    //                     msg
    //                 );
    //             }
    //             Err(e) => {
    //                 println!(
    //                     "[Node RT] ERROR: Failed to receive message from orchestrator: {}",
    //                     e
    //                 );
    //             }
    //         }
    //     }

    //     // Keep the runtime running indefinitely
    //     loop {
    //         tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    //     }

    //     Ok(())
    // }

    pub async fn run(&mut self, args: Arguments, orch_addr: SocketAddr) -> Result<(), Error> {
        println!(
            "[Node RT] Running as node runtime with flow file: {}",
            args.flow
        );

        let orchestrator_ip = orch_addr.ip().to_string();
        let runtime_id = self.execution_config.runtime_id;
        let (tx, rx) = tokio::sync::oneshot::channel();

        let orch_receiver_shared = Arc::clone(&self.orch_receiver);
        self.spawn_receiver_task(orch_receiver_shared.clone(), RUNTIME_PORT, Some(tx))
            .await;
        NodeRuntime::spawn_r2r_message_handler(
            //r2r_receiver_shared.clone(),
            Arc::clone(&self.executor),
            Arc::clone(&self.node_id_map),
            Arc::clone(&self.abstract_flow),
            self.execution_config.clone(), // we'll fix clone below
            self.orchestrator_ip.clone(),
            Arc::clone(&self.orch_sender),
        );

        let assigned_port = self
            .perform_orchestrator_handshake(&orchestrator_ip, runtime_id, rx)
            .await?;
        self.setup_sender_to_orchestrator(&orchestrator_ip, assigned_port)
            .await?;
        self.send_acknowledgment_to_orchestrator(assigned_port)
            .await?;

        println!(
            "[Node RT] Successfully connected to orchestrator on {}:{} (send) and {}:{} (receive)",
            orchestrator_ip, assigned_port, orchestrator_ip, RUNTIME_PORT
        );

        self.message_loop(assigned_port).await
    }

    async fn spawn_receiver_task(
        &self,
        receiver_shared: Arc<Mutex<NetworkCommunicator<String>>>,
        port: u16,
        tx: Option<tokio::sync::oneshot::Sender<()>>,
    ) {
        let orchestrator_ip = self.orchestrator_ip.clone();
        tokio::spawn(async move {
            let mut receiver_guard = receiver_shared.lock().await;
            if let Some(tx) = tx {
                let _ = tx.send(());
            }
            match receiver_guard
                .connect_recv(Some(orchestrator_ip), Some(port))
                .await
            {
                Ok(_) => println!(
                    "[Node RT] Receiver successfully established on port {}",
                    port
                ),
                Err(e) => println!(
                    "[Node RT] Receiver failed to bind or accept connection: {}",
                    e
                ),
            }
        });
    }

    /// Spawns the R2R message handler that continuously listens on R2R_PORT,
    /// accepts a single connection (from any IP), reads a single message, and
    /// then processes it.
    pub fn spawn_r2r_message_handler(
        executor: Arc<Mutex<StandardExecutor>>,
        node_id_map: Arc<Mutex<HashMap<NodeId, String>>>,
        abstract_flow: Arc<Mutex<AbstractFlow>>,
        execution_config: ExecutionConfig,
        orchestrator_ip: String,
        orch_sender: Arc<Mutex<NetworkCommunicator<String>>>,
    ) {
        tokio::spawn(async move {
            println!(
                "[Node RT] [R2R] Starting R2R message handler on port {}...",
                R2R_PORT
            );
            loop {
                // Use our new method to accept one connection and receive one message.
                match NetworkCommunicator::<String>::listen_single_message_on_fixed_port(R2R_PORT)
                    .await
                {
                    Ok(Message::RequestPeerConnection(
                        sending_node_id,
                        receiving_node_id,
                        sender_out_idx,
                        receiver_in_idx,
                        type_name,
                    )) => {
                        println!(
                            "[Node RT] [R2R] Received RequestPeerConnection: {} → {} ({} → {}) | Type: {}",
                            sending_node_id, receiving_node_id, sender_out_idx, receiver_in_idx, type_name
                        );

                        // Handle the connection setup inline.
                        if let Err(e) = Self::handle_request_peer_connection_static(
                            Arc::clone(&executor),
                            Arc::clone(&node_id_map),
                            Arc::clone(&abstract_flow),
                            execution_config.clone(),
                            Arc::clone(&orch_sender),
                            orchestrator_ip.clone(),
                            sending_node_id,
                            receiving_node_id,
                            sender_out_idx,
                            receiver_in_idx,
                            type_name,
                        )
                        .await
                        {
                            eprintln!(
                                "[Node RT] [R2R] Error in RequestPeerConnection handler: {}",
                                e
                            );
                        }
                    }
                    Ok(msg) => {
                        println!("[Node RT] [R2R] Unhandled message: {:?}", msg);
                    }
                    Err(e) => {
                        eprintln!("[Node RT] [R2R] Error receiving message: {}", e);
                        // Optionally, sleep for a short duration before retrying
                        // tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn handle_request_peer_connection_static(
        executor: Arc<Mutex<StandardExecutor>>,
        node_id_map: Arc<Mutex<HashMap<NodeId, String>>>,
        _abstract_flow: Arc<Mutex<AbstractFlow>>,
        execution_config: ExecutionConfig,
        orch_sender: Arc<Mutex<NetworkCommunicator<String>>>, // not used in this handler
        _orchestrator_ip: String,
        sending_node_id: NodeId,
        receiving_node_id: NodeId,
        sender_out_idx: NodeIOIndex,
        receiver_in_idx: NodeIOIndex,
        type_name: String,
    ) -> Result<(), anyhow::Error> {
        println!(
            "[Node RT] [R2R] Handling RequestPeerConnection: {} -> {} ({} -> {})",
            sending_node_id, receiving_node_id, sender_out_idx, receiver_in_idx
        );

        // Verify node locality
        let is_local = execution_config
            .node_configs
            .get(&receiving_node_id)
            .is_some_and(|cfg| matches!(cfg, NodeConfig::LocalNodeConfig));

        if !is_local {
            println!(
                "[Node RT] [R2R] WARNING: Target node is not local: {}",
                receiving_node_id
            );
            return Ok(()); // Do not error, just skip
        }

        //let executor_guard = executor.lock().await;

        // let node = executor_guard
        //     .execution_nodes
        //     .get(&receiving_node_id)
        //     .ok_or_else(|| anyhow::anyhow!("Target node {} not found", receiving_node_id))?;
        // let node_guard = node.lock().await;

        let sender_ip = Self::resolve_runtime_ip(
            sending_node_id,
            Arc::clone(&node_id_map),
            Arc::clone(&orch_sender),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to retrieve IP: {}", e))?;

        // let sender_ip = node_id_map
        //     .lock()
        //     .await
        //     .get(&sending_node_id)
        //     .ok_or_else(|| anyhow::anyhow!("No runtime IP for sending node {}", sending_node_id))?
        //     .clone();

        let mut boxed_comm = TYPE_REGISTRY
            .lock()
            .await
            .create_communicator_by_name(&type_name)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create communicator: {}", e))?;

        let recv_port = 6000; // TODO: assign dynamically later

        // Create response message
        let response_msg = Message::AcceptPeerConnection(
            sending_node_id,
            receiving_node_id,
            sender_out_idx,
            receiver_in_idx,
            recv_port,
        );

        let mut return_comm = NetworkCommunicator::<String>::new()
            .await
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
        return_comm
            .connect_send(Some(sender_ip.clone()), Some(R2R_PORT))
            .await
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;

        println!(
            "[Node RT] [R2R] Sending AcceptPeerConnection to {} on port {}",
            sender_ip, RUNTIME_PORT
        );
        return_comm
            .send(response_msg)
            .await
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;

        // Spawn receive-side listener
        let type_name_clone = type_name.clone();
        let executor_clone = Arc::clone(&executor);
        tokio::spawn(async move {
            let mut attempt = 0;
            let max_delay = Duration::from_secs(10);

            loop {
                match boxed_comm.connect_receive(recv_port).await {
                    Ok(_) => match boxed_comm.into_node_communicator() {
                        Ok(node_comm) => {
                            let exec = executor_clone.lock().await;
                            if let Some(node) = exec.execution_nodes.get(&receiving_node_id) {
                                let mut guard = node.lock().await;
                                if let Err(e) = TYPE_REGISTRY.lock().await.set_input_comm(
                                    &type_name_clone,
                                    guard.get_io_mut(),
                                    receiver_in_idx,
                                    node_comm,
                                ) {
                                    eprintln!(
                                        "[Node RT] [R2R] Failed to inject communicator: {}",
                                        e
                                    );
                                } else {
                                    println!("[Node RT] [R2R] Receiver side established for {} -> {} (port {})",
                                    sending_node_id, receiving_node_id, recv_port);
                                }
                            }
                            break;
                        }
                        Err(e) => {
                            eprintln!("[Node RT] [R2R] Failed to convert communicator: {}", e);
                            break;
                        }
                    },
                    Err(e) => {
                        attempt += 1;
                        let delay = std::cmp::min(
                            Duration::from_millis(100 * 2u64.pow(attempt.min(6))),
                            max_delay,
                        );
                        eprintln!("[Node RT] [R2R] Attempt {} failed to bind receiver: {}. Retrying in {:?}...", attempt, e, delay);
                        sleep(delay).await;
                    }
                }
            }
        });

        Ok(())
    }

    async fn perform_orchestrator_handshake(
        &mut self,
        orchestrator_ip: &str,
        runtime_id: u128,
        rx: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<u16, Error> {
        let mut retries = 0;
        while retries < 10 {
            match TcpStream::connect((orchestrator_ip, SETUP_PORT)).await {
                Ok(mut stream) => {
                    println!(
                        "[Node RT] Successfully signaled presence to orchestrator at {}:{}",
                        orchestrator_ip, SETUP_PORT
                    );
                    let id_msg = format!("RUNTIME_ID:{}", runtime_id);
                    stream.write_all(id_msg.as_bytes()).await?;
                    break;
                }
                Err(_) => {
                    println!("[Node RT] Failed to connect to orchestrator setup port. Retrying... ({}/10)", retries + 1);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    retries += 1;
                }
            }
        }

        if rx.await.is_err() {
            return Err(anyhow::anyhow!("Receiver setup failed"));
        }

        let receiver_shared = Arc::clone(&self.orch_receiver);
        let mut receiver_guard = receiver_shared.lock().await;
        match timeout(
            Duration::from_secs(2),
            Communicator::<String>::receive(&mut *receiver_guard),
        )
        .await
        {
            Ok(Ok(Message::SetupCommunicationPort(port))) => {
                self.assigned_port = Some(port);
                Ok(port)
            }
            Ok(Ok(_)) => Err(anyhow::anyhow!("Unexpected message received")),
            Ok(Err(e)) => Err(anyhow::anyhow!("Receive error: {}", e)),
            Err(_) => Err(anyhow::anyhow!(
                "Timed out waiting for SetupCommunicationPort"
            )),
        }
    }

    async fn setup_sender_to_orchestrator(
        &self,
        orchestrator_ip: &str,
        port: u16,
    ) -> Result<(), Error> {
        let mut sender_guard = self.orch_sender.lock().await;
        *sender_guard = NetworkCommunicator::new()
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        sender_guard
            .connect_send(Some(orchestrator_ip.to_string()), Some(port))
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        println!(
            "[Node RT] Sender established. Sending messages to orchestrator at {}:{}",
            orchestrator_ip, port
        );
        Ok(())
    }

    async fn send_acknowledgment_to_orchestrator(&self, port: u16) -> Result<(), Error> {
        let msg = Message::<String>::AcknowledgeConnection;
        println!(
            "[Node RT] Sending acknowledgment message to orchestrator on port {}...",
            port
        );
        let mut sender_guard = self.orch_sender.lock().await;
        sender_guard
            .send(msg)
            .await
            .map_err(|e| anyhow::anyhow!("Ack send error: {}", e))
    }

    async fn message_loop(&mut self, assigned_port: u16) -> Result<(), Error> {
        let receiver_shared = Arc::clone(&self.orch_receiver);
        loop {
            let mut receiver_guard = receiver_shared.lock().await;
            println!(
                "[Node RT] Waiting for InitializeLocalNodes on port {}",
                assigned_port
            );
            match Communicator::<String>::receive(&mut *receiver_guard).await {
                Ok(Message::InitializeLocalNodes) => self.handle_initialize_local_nodes().await?,
                Ok(Message::OrchestratorRequestNodeConnection(
                    sender,
                    recv,
                    rt_id,
                    ip,
                    out_idx,
                    in_idx,
                )) => {
                    self.handle_orchestrator_request_connection(
                        sender, recv, rt_id, ip, out_idx, in_idx,
                    )
                    .await?
                }
                Ok(Message::StartExecution) => self.handle_start_execution().await?,
                // Ok(Message::RequestNodeRuntimeIP(id)) => {
                //     self.handle_request_node_runtime_ip(id).await
                // }
                Ok(Message::RespondNodeRuntimeIP(id, ip)) => {
                    self.handle_respond_node_runtime_ip(id, ip).await;
                }
                // Ok(Message::RequestPeerConnection(sender, recv, out_idx, in_idx, t)) => {
                //     self.handle_request_peer_connection(sender, recv, out_idx, in_idx, t)
                //         .await?
                // }
                // Ok(Message::AcceptPeerConnection(sender, recv, out_idx, in_idx, port)) => {
                //     self.handle_accept_peer_connection(sender, recv, out_idx, in_idx, port)
                //         .await?
                // }
                Ok(m) => println!("[Node RT] WARNING: Unexpected message: {:?}", m),
                Err(e) => println!("[Node RT] ERROR: Message receive failed: {}", e),
            }
        }
    }
    /// Handles the incoming P2P connection request from the orchestrator
    async fn handle_p2p_connection(
        &mut self,
        sender_id: NodeId,
        receiver_id: NodeId,
        sender_out_idx: u128,
        recv_in_idx: u128,
        receiver_runtime_ip: String,
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

        // Step 3A-1: Retrieve the types of the sender and receiver nodes
        let (out_type_name, out_type_id) = self
            .abstract_flow
            .lock()
            .await
            .get_output_type(sender_id, sender_out_idx)
            .ok_or_else(|| {
                anyhow::Error::msg(format!(
                    "Type not found: sender out index {}",
                    sender_out_idx
                ))
            })?;

        let (_in_type_name, in_type_id) = self
            .abstract_flow
            .lock()
            .await
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

        if is_local {
            println!(
                "[Node RT] Establishing local connection between nodes {} and {}...",
                sender_id, receiver_id
            );

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

                // println!(
                //     "[DEBUG] Raw sender IO type: {:?}",
                //     sender_guard.node.get_io_mut()
                // );
                // println!(
                //     "[DEBUG] Raw receiver IO type: {:?}",
                //     receiver_guard.node.get_io_mut().type_id()
                // );

                let sender_io = sender_guard.node.get_io_mut();
                let receiver_io = receiver_guard.node.get_io_mut();

                //let sender_io = get_concrete_io(&mut sender_guard.node).as_any_mut();
                //let receiver_io = get_concrete_io(&mut receiver_guard.node).as_any_mut();

                // Call the dynamic connection function directly
                Self::connect_nodes_dynamic(
                    out_type_id,
                    sender_id,
                    receiver_id,
                    sender_out_idx,
                    recv_in_idx,
                    sender_io,
                    receiver_io,
                )
                .await;
            }

            println!(
                "[Node RT] Successfully connected local nodes {} -> {}",
                sender_id, receiver_id
            );

            // send acknoweldge message for local connections
            self.send_acknowledge_connection(sender_id, receiver_id, sender_out_idx, recv_in_idx)
                .await?;
        } else {
            println!(
                "[Node RT] Establishing remote connection between nodes {} and {}...",
                sender_id, receiver_id
            );

            let mut setup_comm = NetworkCommunicator::<String>::new().await.unwrap();
            setup_comm
                .connect_send(Some(receiver_runtime_ip.clone()), Some(R2R_PORT))
                .await
                .map_err(|e| {
                    anyhow::Error::msg(format!(
                        "Failed to connect to receiver runtime {}: {}",
                        receiver_runtime_ip, e
                    ))
                })?;

            // Step 3: Build and send request message
            let request_message = Message::<String>::RequestPeerConnection(
                sender_id,
                receiver_id,
                sender_out_idx,
                recv_in_idx,
                out_type_name,
            );

            setup_comm.send(request_message).await.map_err(|e| {
                anyhow::Error::msg(format!(
                    "Failed to send RequestPeerConnection to {}: {}",
                    receiver_runtime_ip, e
                ))
            })?;

            println!(
                "[Node RT] Sent RequestPeerConnection to {}. Waiting for peer port...",
                receiver_runtime_ip
            );
        }

        Ok(())
    }

    async fn connect_nodes_dynamic(
        type_id: TypeId,
        sender_id: NodeId,
        receiver_id: NodeId,
        sender_out_idx: NodeIOIndex,
        recv_in_idx: NodeIOIndex,
        sender_io: &mut dyn SetupIO,
        receiver_io: &mut dyn SetupIO,
    ) {
        if let Some(connect_fn) = TYPE_REGISTRY.lock().await.get(type_id) {
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

    async fn send_acknowledge_connection(
        &self,
        sending_node_id: NodeId,
        receiving_node_id: NodeId,
        sender_out_idx: NodeIOIndex,
        receiver_in_idx: NodeIOIndex,
    ) -> Result<(), anyhow::Error> {
        let max_attempts = 5;
        let mut attempt = 0;

        loop {
            attempt += 1;

            println!(
                "[ACK] Attempt {}/{}: Sending AcknowledgeConnectionSetup...",
                attempt, max_attempts
            );

            let result = {
                let mut sender_guard = self.orch_sender.lock().await;

                sender_guard
                    .send(Message::AcknowledgeConnectionSetup(
                        sending_node_id,
                        receiving_node_id,
                        sender_out_idx,
                        receiver_in_idx,
                    ))
                    .await
            };

            match result {
                Ok(_) => {
                    println!("[ACK] Successfully sent acknowledgment.");
                    break Ok(());
                }
                Err(e) if attempt < max_attempts => {
                    println!(
                        "[ACK] Failed to send acknowledgment on attempt {}: {}. Retrying...",
                        attempt, e
                    );
                    let backoff = Duration::from_millis(200 * attempt);
                    sleep(backoff).await;
                }
                Err(e) => {
                    break Err(anyhow::anyhow!(
                        "Failed to send acknowledgment after {} attempts: {}",
                        attempt,
                        e
                    ));
                }
            }
        }
    }

    /////////////////////////////////////////////////////////////
    /// MESSAGE HANDLER FUNCTIONS
    ////////////////////////////////////////////////////////////

    pub async fn handle_initialize_local_nodes(&mut self) -> Result<(), anyhow::Error> {
        println!("[Node RT] Received InitializeLocalNodes request...");
        let executor = Arc::clone(&self.executor);
        let mut executor_guard = executor.lock().await;
        // Perform actual node initialization
        if let Err(e) = executor_guard
            .initialize_nodes(Arc::clone(&self.abstract_flow), &self.execution_config)
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
        {
            let mut sender_guard = self.orch_sender.lock().await;
            if let Err(e) = sender_guard.send(ack_message).await {
                println!(
                    "[Node RT] ERROR: Failed to send AcknowledgeNodeInitialization: {}",
                    e
                );
            } else {
                println!("[Node RT] Sent AcknowledgeNodeInitialization.");
            }
        }

        Ok(())
    }

    pub async fn handle_orchestrator_request_connection(
        &mut self,
        sender_id: NodeId,
        receiver_id: NodeId,
        _runtime_id: u128,
        runtime_ip: String,
        sender_out_idx: NodeIOIndex,
        recv_in_idx: NodeIOIndex,
    ) -> Result<(), anyhow::Error> {
        println!(
            "[Node RT] Received P2P connection request: {} -> {} (Out {} -> In {})",
            sender_id, receiver_id, sender_out_idx, recv_in_idx
        );

        self.node_id_map
            .lock()
            .await
            .insert(receiver_id, runtime_ip.clone());

        let connection = FlowNodeConnection {
            sender_id,
            receiver_id,
            send_out_idx: sender_out_idx,
            recv_in_idx,
        };

        let connection_type = self
            .abstract_flow
            .lock()
            .await
            .get_connection_type(&connection);
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
        // &self
        //     .execution_config
        //     .is_local_connection(sender_id, receiver_id);

        //let runtime = Arc::new(Mutex::new(self));
        if let Err(e) = self
            .handle_p2p_connection(
                sender_id,
                receiver_id,
                sender_out_idx,
                recv_in_idx,
                runtime_ip,
            )
            .await
        {
            println!("[Node RT] ERROR: Failed to setup P2P connection: {}", e);
        }

        Ok(())
    }

    pub async fn handle_start_execution(&mut self) -> Result<(), anyhow::Error> {
        println!("[Node RT] Received StartExecution command. Beginning execution...");

        if let Err(e) = self.start_execution().await {
            println!("[Node RT] ERROR: Execution failed: {}", e);
        } else {
            println!("[Node RT] Execution completed successfully.");
        }

        Ok(())
    }

    pub async fn handle_request_node_runtime_ip(&mut self, node_id: NodeId) {
        println!(
            "[Node RT] Received request for Node {}'s IP from Orchestrator...",
            node_id
        );

        // Check if we already have the IP stored
        if let Some(ip) = self.node_id_map.lock().await.get(&node_id) {
            println!("[Node RT] Sending cached IP for Node {}: {}", node_id, ip);
            let response = Message::<String>::RespondNodeRuntimeIP(node_id, ip.clone());
            let mut sender_guard = self.orch_sender.lock().await;
            if let Err(e) = sender_guard.send(response).await {
                println!("[Node RT] ERROR: Failed to send IP response: {}", e);
            }
        } else {
            println!("[Node RT] WARNING: No known IP for Node {}!", node_id);
        }
    }

    pub async fn handle_respond_node_runtime_ip(&mut self, node_id: NodeId, ip: String) {
        println!("[Node RT] Received IP for Node {}: {}", node_id, ip);
        self.node_id_map.lock().await.insert(node_id, ip);
    }

    // pub async fn handle_request_peer_connection(
    //     &mut self,
    //     sending_node_id: NodeId,
    //     receiving_node_id: NodeId,
    //     sender_out_idx: NodeIOIndex,
    //     receiver_in_idx: NodeIOIndex,
    //     type_name: String,
    // ) -> Result<(), anyhow::Error> {
    //     println!(
    //     "[Node RT] Received RequestPeerConnection from node {} (output idx: {}) -> {} (input idx: {})",
    //     sending_node_id, sender_out_idx, receiving_node_id, receiver_in_idx
    // );

    //     // let is_local = self
    //     //     .execution_config
    //     //     .node_configs
    //     //     .get(&receiving_node_id)
    //     //     .is_some_and(|config| match config {
    //     //         NodeConfig::LocalNodeConfig => true,
    //     //         _ => false,
    //     //     });

    //     //             if !is_local {
    //     //                 println!(
    //     //     "[Node RT] WARNING: Received peer connection request for non-local node {}",
    //     //     receiving_node_id
    //     // );
    //     //                 continue;
    //     //             }

    //     //let executor = Arc::clone(&self.executor);
    //     //let mut executor_guard = executor.lock().await;

    //     // 1. Get the receiving ExecutionNode
    //     //let node = executor_guard
    //     //    .execution_nodes
    //     //    .get(&receiving_node_id)
    //     //    .ok_or_else(|| anyhow::anyhow!("Target node {} not found", receiving_node_id))?;
    //     //let mut node_guard = node.lock().await;

    //     // 2. Get sender runtime IP (for reply)
    //     let sending_runtime_ip = self
    //         .node_id_map
    //         .lock()
    //         .await
    //         .get(&sending_node_id)
    //         .ok_or_else(|| {
    //             anyhow::anyhow!("No runtime IP found for sending node {}", sending_node_id)
    //         })?
    //         .clone();

    //     // 3. Create NetworkCommunicator via type registry
    //     let mut boxed_comm = TYPE_REGISTRY
    //         .lock()
    //         .await
    //         .create_communicator_by_name(&type_name)
    //         .await
    //         .map_err(|e| anyhow::anyhow!("Failed to create communicator: {}", e))?;

    //     // 4. Choose a port and bind to it
    //     let recv_port = 6000; // TODO: dynamic port assignment

    //     // Compose the message
    //     let response_msg = Message::AcceptPeerConnection(
    //         sending_node_id,
    //         receiving_node_id,
    //         sender_out_idx,
    //         receiver_in_idx,
    //         recv_port,
    //     );

    //     // 7. Send back AcceptPeerConnection message
    //     // Create a communicator to send back the AcceptPeerConnection
    //     let mut return_comm = NetworkCommunicator::<String>::new()
    //         .await
    //         .map_err(|e| anyhow::anyhow!("Failed to create return communicator: {}", e))?;

    //     println!(
    //         "[Node RT] Sending AcceptPeerConnection message with port {}...",
    //         recv_port
    //     );
    //     return_comm
    //         .connect_send(Some(sending_runtime_ip.clone()), Some(R2R_PORT))
    //         .await
    //         .map_err(|e| anyhow::anyhow!("Failed to connect to sender runtime: {}", e))?;
    //     println!("[Node RT] AcceptPeerConnection message sent!");

    //     // 4. Spawn task to bind, wait, and inject the communicator
    //     let type_name = type_name.clone();
    //     let executor = Arc::clone(&self.executor);
    //     let executor_clone = Arc::clone(&executor);

    //     tokio::spawn(async move {
    //         let mut attempt: u32 = 0;
    //         let max_delay = Duration::from_secs(10); // max delay between retries

    //         loop {
    //             match boxed_comm.connect_receive(recv_port).await {
    //                 Ok(()) => {
    //                     match boxed_comm.into_node_communicator() {
    //                         Ok(node_comm) => {
    //                             let executor_guard = executor_clone.lock().await;
    //                             let node =
    //                                 match executor_guard.execution_nodes.get(&receiving_node_id) {
    //                                     Some(node) => node,
    //                                     None => {
    //                                         eprintln!(
    //                                             "[Node RT] ERROR: Node {} not found",
    //                                             receiving_node_id
    //                                         );
    //                                         return;
    //                                     }
    //                                 };
    //                             let mut node_guard = node.lock().await;

    //                             if let Err(e) = TYPE_REGISTRY.lock().await.set_input_comm(
    //                                 &type_name,
    //                                 node_guard.get_io_mut(),
    //                                 receiver_in_idx,
    //                                 node_comm,
    //                             ) {
    //                                 eprintln!(
    //                                                 "[Node RT] ERROR: Failed to inject communicator for {} -> {}: {}",
    //                                                 sending_node_id, receiving_node_id, e
    //                                             );
    //                             } else {
    //                                 println!(
    //                                                 "[Node RT] Receiver side established for {} -> {} (port {})",
    //                                                 sending_node_id, receiving_node_id, recv_port
    //                                             );
    //                             }

    //                             break; // done
    //                         }
    //                         Err(e) => {
    //                             eprintln!(
    //                                 "[Node RT] ERROR: Failed to convert to node communicator: {}",
    //                                 e
    //                             );
    //                             break;
    //                         }
    //                     }
    //                 }
    //                 Err(err) => {
    //                     attempt += 1;
    //                     let delay = std::cmp::min(
    //                         Duration::from_millis(100 * 2u64.pow(attempt.min(6))), // exponential backoff
    //                         max_delay,
    //                     );
    //                     eprintln!(
    //                                     "[Node RT] WARN: Attempt {} to bind receiver on port {} failed: {}. Retrying in {:?}...",
    //                                     attempt, recv_port, err, delay
    //                                 );
    //                     tokio::time::sleep(delay).await;
    //                 }
    //             }
    //         }
    //     });

    //     // Send the message using the communicator
    //     return_comm
    //         .send(response_msg)
    //         .await
    //         .map_err(|e| anyhow::anyhow!("Failed to send AcceptPeerConnection message: {}", e))?;

    //     Ok(())
    // }

    pub async fn handle_accept_peer_connection(
        &mut self,
        sending_node_id: NodeId,
        receiving_node_id: NodeId,
        sender_out_idx: NodeIOIndex,
        receiver_in_idx: NodeIOIndex,
        recv_port: u16,
    ) -> Result<(), anyhow::Error> {
        let executor = Arc::clone(&self.executor);
        let executor_guard = executor.lock().await;

        // 1. Get the sending ExecutionNode
        let node = executor_guard
            .execution_nodes
            .get(&sending_node_id)
            .ok_or_else(|| anyhow::anyhow!("Sending node {} not found", sending_node_id))?;
        let mut node_guard = node.lock().await;

        // 2. Get the receiving node's IP
        let receiving_runtime_ip = {
            const MAX_RETRIES: usize = 10;
            const RETRY_DELAY_MS: u64 = 100;

            let mut attempts = 0;
            loop {
                let ip_opt = self
                    .node_id_map
                    .lock()
                    .await
                    .get(&receiving_node_id)
                    .cloned();

                if let Some(ip) = ip_opt {
                    break Ok(ip);
                }

                if attempts >= MAX_RETRIES {
                    break Err(anyhow!(
                        "Timeout while waiting for IP of node {} after {} attempts",
                        receiving_node_id,
                        MAX_RETRIES
                    ));
                }

                attempts += 1;
                sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
            }
        }?;

        // 3. Get the type name
        let (type_name, _type_id) = self
            .abstract_flow
            .lock()
            .await
            .get_output_type(sending_node_id, sender_out_idx)
            .ok_or_else(|| {
                anyhow::Error::msg(format!(
                    "Type not found: sender out index {}",
                    sender_out_idx
                ))
            })?;

        // 4. Use the registry to create, connect, and inject the communicator
        TYPE_REGISTRY
            .lock()
            .await
            .set_output_comm_with_connection(
                &type_name,
                node_guard.get_io_mut(),
                sender_out_idx,
                receiving_runtime_ip,
                recv_port,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to set output communicator: {}", e))?;

        // 5. Send AcknowledgeConnectionSetup to orchestrator
        self.send_acknowledge_connection(
            sending_node_id,
            receiving_node_id,
            sender_out_idx,
            receiver_in_idx,
        )
        .await?;

        Ok(())
    }

    async fn resolve_runtime_ip(
        node_id: NodeId,
        node_id_map: Arc<Mutex<HashMap<NodeId, String>>>,
        orch_sender: Arc<Mutex<NetworkCommunicator<String>>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let start = tokio::time::Instant::now();
        let timeout = Duration::from_secs(10);

        loop {
            {
                let map = node_id_map.lock().await;
                if let Some(ip) = map.get(&node_id) {
                    return Ok(ip.clone());
                }
            }

            {
                let mut sender = orch_sender.lock().await;
                sender.send(Message::RequestNodeRuntimeIP(node_id)).await?;
            }

            if start.elapsed() > timeout {
                return Err("Timed out waiting for runtime IP".into());
            }

            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
}
