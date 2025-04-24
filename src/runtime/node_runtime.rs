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
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task::yield_now;
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
    node_id_map: Arc<Mutex<HashMap<NodeId, String>>>,
    next_port: Arc<Mutex<u16>>,
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
        let executor = Arc::new(Mutex::new(StandardExecutor::new()));
        let node_id_map = Arc::new(Mutex::new(HashMap::<NodeId, String>::new()));
        let abstract_flow = Arc::new(Mutex::new(abstract_flow));
        let next_port = Arc::new(Mutex::new(6000u16));

        Ok(Self {
            abstract_flow,
            orchestrator_ip,
            _runtime_id,
            assigned_port: None,
            execution_config,
            executor,
            orch_sender,
            orch_receiver,
            node_id_map,
            next_port,
        })
    }

    pub fn clone_runtime_handle(&self) -> Self {
        Self {
            abstract_flow: Arc::clone(&self.abstract_flow),
            orchestrator_ip: self.orchestrator_ip.clone(),
            _runtime_id: self._runtime_id,
            assigned_port: self.assigned_port,
            execution_config: self.execution_config.clone(),
            executor: Arc::clone(&self.executor),
            orch_sender: Arc::clone(&self.orch_sender),
            orch_receiver: Arc::clone(&self.orch_receiver),
            node_id_map: Arc::clone(&self.node_id_map),
            next_port: Arc::clone(&self.next_port),
        }
    }

    pub async fn run(&mut self, args: Arguments, orch_addr: SocketAddr) -> Result<(), Error> {
        tracing::debug!(
            "[Node RT] Running as node runtime with flow file: {}",
            args.flow
        );

        let orchestrator_ip = orch_addr.ip().to_string();
        let runtime_id = self.execution_config.runtime_id;
        let (tx, rx) = tokio::sync::oneshot::channel();

        // Start orchestrator message receiver
        let orch_receiver_shared = Arc::clone(&self.orch_receiver);
        self.spawn_receiver_task(orch_receiver_shared.clone(), RUNTIME_PORT, Some(tx))
            .await;

        // Only clone Arc for R2R handler here — we don't consume `self`
        let runtime_arc = Arc::new(self.clone_runtime_handle());
        Self::spawn_r2r_message_handler(runtime_arc);

        // Handshake and setup
        let assigned_port = self
            .perform_orchestrator_handshake(&orchestrator_ip, runtime_id, rx)
            .await?;
        self.setup_sender_to_orchestrator(&orchestrator_ip, assigned_port)
            .await?;
        self.send_acknowledgment_to_orchestrator(assigned_port)
            .await?;

        tracing::debug!(
            "[Node RT] Successfully connected to orchestrator on {}:{} (send) and {}:{} (receive)",
            orchestrator_ip,
            assigned_port,
            orchestrator_ip,
            RUNTIME_PORT
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
                Ok(_) => tracing::debug!(
                    "[Node RT] Receiver successfully established on port {}",
                    port
                ),
                Err(e) => tracing::debug!(
                    "[Node RT] Receiver failed to bind or accept connection: {}",
                    e
                ),
            }
        });
    }

    /// Spawns the R2R message handler that continuously listens on R2R_PORT,
    /// accepts a single connection (from any IP), reads a single message, and
    /// then processes it.
    /// Spawns the R2R message handler that continuously listens on R2R_PORT,
    /// accepts a single connection (from any IP), reads a single message, and then processes it.
    pub fn spawn_r2r_message_handler(runtime: Arc<Self>) {
        tokio::spawn(async move {
            tracing::debug!(
                "[Node RT] [R2R] Starting R2R message handler on port {}...",
                R2R_PORT
            );

            let listener = match TcpListener::bind(("0.0.0.0", R2R_PORT)).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("[Node RT] [R2R] Failed to bind to port {}: {}", R2R_PORT, e);
                    return;
                }
            };

            loop {
                match listener.accept().await {
                    Ok((mut socket, addr)) => {
                        tracing::debug!("[NetworkCommunicator] Accepted R2R message from {}", addr);

                        let mut line = String::new();
                        let mut buffer = [0; 1];
                        let mut buf_reader: BufReader<&mut TcpStream> = BufReader::new(&mut socket);

                        let result = match buf_reader.get_ref().peek(&mut buffer).await {
                            Ok(n) if n > 0 => {
                                if buf_reader.read_line(&mut line).await.is_ok() {
                                    Message::<String>::from_str(&line)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        };

                        // Clone inside the match arm to avoid move errors
                        let executor = Arc::clone(&runtime.executor);
                        let node_id_map = Arc::clone(&runtime.node_id_map);
                        let abstract_flow = Arc::clone(&runtime.abstract_flow);
                        let execution_config = runtime.execution_config.clone();
                        let orch_sender = Arc::clone(&runtime.orch_sender);
                        let orchestrator_ip = runtime.orchestrator_ip.clone();
                        let next_port = Arc::clone(&runtime.next_port);

                        match result {
                            Some(Message::RequestPeerConnection(
                                sending_node_id,
                                receiving_node_id,
                                sender_out_idx,
                                receiver_in_idx,
                                type_name,
                            )) => {
                                tracing::debug!(
                                "[Node RT] [R2R] Received RequestPeerConnection: {} → {} ({} → {}) | Type: {}",
                                sending_node_id, receiving_node_id, sender_out_idx, receiver_in_idx, type_name
                            );

                                // tokio::spawn(async move {
                                //     if let Err(e) = handle_request_peer_connection_static(
                                //         executor,
                                //         node_id_map,
                                //         abstract_flow,
                                //         execution_config,
                                //         orch_sender,
                                //         orchestrator_ip,
                                //         sending_node_id,
                                //         receiving_node_id,
                                //         sender_out_idx,
                                //         receiver_in_idx,
                                //         type_name,
                                //         next_port,
                                //     )
                                //     .await
                                //     {
                                //         eprintln!("[Node RT] [R2R] Error: {}", e);
                                //     }
                                // });
                                tokio::spawn(handle_peer_connection_spawn(
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
                                    Arc::clone(&next_port),
                                ));
                            }

                            Some(Message::AcceptPeerConnection(
                                sending_node_id,
                                receiving_node_id,
                                sender_out_idx,
                                receiver_in_idx,
                                recv_port,
                            )) => {
                                tracing::debug!(
                                "[Node RT] [R2R] Received AcceptPeerRequest: {} → {} ({} → {}) | Port: {}",
                                sending_node_id, receiving_node_id, sender_out_idx, receiver_in_idx, recv_port
                            );

                                tokio::spawn(async move {
                                    if let Err(e) = handle_accept_peer_connection_static(
                                        executor,
                                        node_id_map,
                                        abstract_flow,
                                        orch_sender,
                                        sending_node_id,
                                        receiving_node_id,
                                        sender_out_idx,
                                        receiver_in_idx,
                                        recv_port,
                                    )
                                    .await
                                    {
                                        eprintln!("[Node RT] [R2R] Error: {}", e);
                                    }
                                });
                            }

                            Some(msg) => {
                                tracing::debug!("[Node RT] [R2R] Unhandled message: {:?}", msg);
                            }

                            None => {
                                eprintln!("[Node RT] [R2R] Failed to parse or read message.");
                            }
                        }
                    }

                    Err(e) => {
                        eprintln!("[Node RT] [R2R] Error accepting connection: {}", e);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }

                yield_now().await;
            }
        });
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
                    tracing::debug!(
                        "[Node RT] Successfully signaled presence to orchestrator at {}:{}",
                        orchestrator_ip,
                        SETUP_PORT
                    );
                    let id_msg = format!("RUNTIME_ID:{}", runtime_id);
                    stream.write_all(id_msg.as_bytes()).await?;
                    break;
                }
                Err(_) => {
                    tracing::debug!("[Node RT] Failed to connect to orchestrator setup port. Retrying... ({}/10)", retries + 1);
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
        tracing::debug!(
            "[Node RT] Sender established. Sending messages to orchestrator at {}:{}",
            orchestrator_ip,
            port
        );
        Ok(())
    }

    async fn send_acknowledgment_to_orchestrator(&self, port: u16) -> Result<(), Error> {
        let msg = Message::<String>::AcknowledgeConnection;
        tracing::debug!(
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
                Ok(Message::RespondNodeRuntimeIP(id, ip)) => {
                    self.handle_respond_node_runtime_ip(id, ip).await?;
                }
                Ok(m) => tracing::debug!("[Node RT] WARNING: Unexpected message: {:?}", m),
                Err(e) => tracing::debug!("[Node RT] ERROR: Message receive failed: {}", e),
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
        tracing::debug!(
            "[Node RT] Handling P2P connection request: {} -> {} (Out {} -> In {})",
            sender_id,
            receiver_id,
            sender_out_idx,
            recv_in_idx
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
            tracing::debug!(
                "[Node RT] Establishing local connection between nodes {} and {}...",
                sender_id,
                receiver_id
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

                // tracing::debug!(
                //     "[DEBUG] Raw sender IO type: {:?}",
                //     sender_guard.node.get_io_mut()
                // );
                // tracing::debug!(
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

            tracing::debug!(
                "[Node RT] Successfully connected local nodes {} -> {}",
                sender_id,
                receiver_id
            );

            // send acknoweldge message for local connections
            self.send_acknowledge_connection(sender_id, receiver_id, sender_out_idx, recv_in_idx)
                .await?;
        } else {
            tracing::debug!(
                "[Node RT] Establishing remote connection between nodes {} and {}...",
                sender_id,
                receiver_id
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

            tracing::debug!(
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
        let connect_fn_option = {
            let registry = TYPE_REGISTRY.lock().await;
            registry.get(type_id).copied()
        };

        if let Some(connect_fn) = connect_fn_option {
            tracing::debug!("[DEBUG] Retrieved type ID for connection: {:?}", type_id);

            connect_fn(
                sender_id,
                receiver_id,
                sender_out_idx,
                recv_in_idx,
                sender_io,
                receiver_io,
            );
            tracing::debug!(
                "[Node RT] Successfully connected nodes using the dynamic function lookup."
            );
        } else {
            panic!(
                "[Node RT] No connection function found for type ID {:?}",
                type_id
            );
        }
    }

    async fn start_execution(&self) -> Result<(), anyhow::Error> {
        tracing::debug!("[Node RT] Starting execution of local nodes...");
        let executor = Arc::clone(&self.executor);
        let executor_guard = executor.lock().await;
        // Make sure nodes are ready
        if let Err(e) = executor_guard.ready_nodes().await {
            tracing::debug!("[Node RT] ERROR: Node readiness failed: {}", e);
            return Err(e);
        }

        // Start execution through executor
        executor_guard.start_execution().await;
        tracing::debug!("[Node RT] Execution started successfully.");
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

            tracing::debug!(
                "[ACK] Attempt {}/{}: Sending AcknowledgeConnectionSetup...",
                attempt,
                max_attempts
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
                    tracing::debug!("[ACK] Successfully sent acknowledgment.");
                    break Ok(());
                }
                Err(e) if attempt < max_attempts => {
                    tracing::debug!(
                        "[ACK] Failed to send acknowledgment on attempt {}: {}. Retrying...",
                        attempt,
                        e
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
        tracing::debug!("[Node RT] Received InitializeLocalNodes request...");
        let executor = Arc::clone(&self.executor);
        let mut executor_guard = executor.lock().await;
        // Perform actual node initialization
        if let Err(e) = executor_guard
            .initialize_nodes(Arc::clone(&self.abstract_flow), &self.execution_config)
            .await
        {
            tracing::debug!("[Node RT] ERROR: Failed to initialize nodes: {}", e);
            return Err(anyhow::Error::msg("Node initialization failed"));
        }

        tracing::debug!("[Node RT] Successfully initialized local nodes.");
        tracing::debug!("[Node RT] All nodes are ready!");

        // Send acknowledgment back to the orchestrator
        let ack_message = Message::<String>::AcknowledgeNodeInitialization;
        {
            let mut sender_guard = self.orch_sender.lock().await;
            if let Err(e) = sender_guard.send(ack_message).await {
                tracing::debug!(
                    "[Node RT] ERROR: Failed to send AcknowledgeNodeInitialization: {}",
                    e
                );
            } else {
                tracing::debug!("[Node RT] Sent AcknowledgeNodeInitialization.");
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
        tracing::debug!(
            "[Node RT] Received P2P connection request: {} -> {} (Out {} -> In {})",
            sender_id,
            receiver_id,
            sender_out_idx,
            recv_in_idx
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
            tracing::debug!(
                "[Node RT] WARNING: P2P request does not match any known connection: {} -> {}",
                sender_id,
                receiver_id
            );
            return Err(anyhow::Error::msg("Invalid P2P connection request"));
        }

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
            tracing::debug!("[Node RT] ERROR: Failed to setup P2P connection: {}", e);
        }

        Ok(())
    }

    pub async fn handle_start_execution(&mut self) -> Result<(), anyhow::Error> {
        tracing::debug!("[Node RT] Received StartExecution command. Beginning execution...");

        if let Err(e) = self.start_execution().await {
            tracing::debug!("[Node RT] ERROR: Execution failed: {}", e);
        } else {
            tracing::debug!("[Node RT] Execution completed successfully.");
        }

        Ok(())
    }

    pub async fn handle_request_node_runtime_ip(&mut self, node_id: NodeId) {
        tracing::debug!(
            "[Node RT] Received request for Node {}'s IP from Orchestrator...",
            node_id
        );

        // Check if we already have the IP stored
        if let Some(ip) = self.node_id_map.lock().await.get(&node_id) {
            tracing::debug!("[Node RT] Sending cached IP for Node {}: {}", node_id, ip);
            let response = Message::<String>::RespondNodeRuntimeIP(node_id, ip.clone());
            let mut sender_guard = self.orch_sender.lock().await;
            if let Err(e) = sender_guard.send(response).await {
                tracing::debug!("[Node RT] ERROR: Failed to send IP response: {}", e);
            }
        } else {
            tracing::debug!("[Node RT] WARNING: No known IP for Node {}!", node_id);
        }
    }

    pub async fn handle_respond_node_runtime_ip(
        &mut self,
        node_id: NodeId,
        ip: String,
    ) -> Result<(), Error> {
        tracing::debug!("[Node RT] Received IP for Node {}: {}", node_id, ip);
        self.node_id_map.lock().await.insert(node_id, ip);
        Ok(())
    }

    pub async fn handle_accept_peer_connection(
        &self,
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
}

pub async fn handle_accept_peer_connection_static(
    executor: Arc<Mutex<StandardExecutor>>,
    node_id_map: Arc<Mutex<HashMap<NodeId, String>>>,
    abstract_flow: Arc<Mutex<AbstractFlow>>,
    orch_sender: Arc<Mutex<NetworkCommunicator<String>>>,
    sending_node_id: NodeId,
    receiving_node_id: NodeId,
    sender_out_idx: NodeIOIndex,
    receiver_in_idx: NodeIOIndex,
    recv_port: u16,
) -> Result<(), anyhow::Error> {
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
            let ip_opt = node_id_map.lock().await.get(&receiving_node_id).cloned();

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
    let (type_name, _type_id) = abstract_flow
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
    let message = Message::<String>::AcknowledgeConnectionSetup(
        sending_node_id,
        receiving_node_id,
        sender_out_idx,
        receiver_in_idx,
    );

    {
        let mut sender = orch_sender.lock().await;
        sender
            .send(message)
            .await
            .map_err(|e| anyhow!("Failed to send acknowledgment: {}", e))?;
    }

    Ok(())
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
    next_port: Arc<Mutex<u16>>,
) -> Result<(), anyhow::Error> {
    tracing::debug!(
        "[Node RT] [R2R] Handling RequestPeerConnection: {} -> {} ({} -> {})",
        sending_node_id,
        receiving_node_id,
        sender_out_idx,
        receiver_in_idx
    );

    // Verify node locality
    let is_local = execution_config
        .node_configs
        .get(&receiving_node_id)
        .is_some_and(|cfg| matches!(cfg, NodeConfig::LocalNodeConfig));

    if !is_local {
        tracing::debug!(
            "[Node RT] [R2R] WARNING: Target node is not local: {}",
            receiving_node_id
        );
        return Ok(());
    }

    tracing::debug!(
        "[Node RT] [R2R] establishing local receiver for node: {}",
        receiving_node_id
    );

    // Resolve sender IP
    let sender_ip = resolve_runtime_ip(
        sending_node_id,
        Arc::clone(&node_id_map),
        Arc::clone(&orch_sender),
    )
    .await
    .map_err(|e| {
        anyhow::anyhow!(
            "Failed to retrieve sender IP for node {}: {}",
            sending_node_id,
            e
        )
    })?;

    tracing::debug!(
        "[Node RT] [R2R] Resolved sender IP for node {}: {}",
        sending_node_id,
        sender_ip
    );
    // Check TypeRegistry Contents
    {
        let registry_guard = TYPE_REGISTRY.lock().await;
        let keys: Vec<_> = registry_guard.name_to_id.keys().cloned().collect();
        tracing::debug!(
            "[Node RT] [R2R] Registry keys before creating communicator: {:?}",
            keys
        );
    } // Drop lock before proceeding to future-based access

    // Create Communicator (Registry lock dropped before await)
    let mut boxed_comm = {
        let registry = TYPE_REGISTRY.lock().await;

        let keys: Vec<_> = registry.name_to_id.keys().cloned().collect();
        tracing::debug!(
            "[DEBUG] Registry keys before create_communicator_by_name: {:?}",
            keys
        );

        registry
            .create_communicator_by_name(&type_name)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create communicator: {}", e))?
    };

    tracing::debug!(
    "[Node RT] [R2R] Successfully created communicator for type '{}' to connect with sender at {}",
    type_name, sender_ip
);
    // Allocate dynamic port for receiving
    let port = {
        let mut port_guard = next_port.lock().await;
        let port = *port_guard;
        *port_guard += 1;
        port
    };

    tracing::debug!("[Node RT] [R2R] Connecting p2p receiver on port {}", port);

    let mut return_comm = NetworkCommunicator::<String>::new()
        .await
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    tracing::debug!(
        "[Node RT] [R2R] Sending AcceptPeerConnection to {} on port {}",
        sender_ip,
        R2R_PORT
    );

    let mut attempt = 0;
    let max_attempts = 10;
    let mut success = false;

    while attempt < max_attempts {
        match return_comm
            .connect_send(Some(sender_ip.clone()), Some(R2R_PORT))
            .await
        {
            Ok(_) => {
                let response_msg = Message::AcceptPeerConnection(
                    sending_node_id,
                    receiving_node_id,
                    sender_out_idx,
                    receiver_in_idx,
                    port,
                );
                if let Err(e) = return_comm.send(response_msg).await {
                    eprintln!(
                        "[Node RT] [R2R] Attempt {}: Failed to send AcceptPeerConnection: {}",
                        attempt + 1,
                        e
                    );
                } else {
                    tracing::debug!(
                        "[Node RT] [R2R] Sent AcceptPeerConnection to {} on port {}",
                        sender_ip,
                        R2R_PORT
                    );
                    success = true;
                    break;
                }
            }
            Err(e) => {
                eprintln!(
                    "[Node RT] [R2R] Attempt {}: Failed to connect to R2R_PORT {}: {}",
                    attempt + 1,
                    R2R_PORT,
                    e
                );
            }
        }

        attempt += 1;
        sleep(Duration::from_millis(100 * 2u64.pow(attempt.min(6)))).await;
    }

    if !success {
        return Err(anyhow::anyhow!(
            "Failed to send AcceptPeerConnection to {} after {} attempts",
            sender_ip,
            max_attempts
        ));
    }
    // Spawn receive-side listener
    let type_name_clone = type_name.clone();
    let executor_clone = Arc::clone(&executor);

    tokio::spawn(async move {
        let mut attempt = 0;
        let max_delay = Duration::from_secs(10);

        loop {
            match boxed_comm
                .connect_receive(sender_ip.clone().as_str(), port)
                .await
            {
                Ok(_) => match boxed_comm.into_node_communicator() {
                    Ok(node_comm) => {
                        let exec = executor_clone.lock().await;
                        if let Some(node) = exec.execution_nodes.get(&receiving_node_id) {
                            let mut guard = node.lock().await;
                            let set_result = {
                                let registry = TYPE_REGISTRY.lock().await;
                                registry.set_input_comm(
                                    &type_name_clone,
                                    guard.get_io_mut(),
                                    receiver_in_idx,
                                    node_comm,
                                )
                            };
                            if let Err(e) = set_result {
                                eprintln!("[Node RT] [R2R] Failed to inject communicator: {}", e);
                            } else {
                                tracing::debug!("[Node RT] [R2R] Receiver side established ...");
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
                    eprintln!(
                        "[Node RT] [R2R] Attempt {} failed to bind receiver on port {}: {}. Retrying in {:?}...",
                        attempt, port, e, delay
                    );
                    sleep(delay).await;
                }
            }
        }
    });

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
                tracing::debug!("[Node RT] Found runtime ip locally: {}", ip.clone());
                return Ok(ip.clone());
            }
        }

        {
            tracing::debug!(
                "[Node RT] Couldn't find runtime ip locally, sending RequestNodeRuntimeIP({})",
                node_id.clone()
            );
            let mut sender = orch_sender.lock().await;
            match sender.send(Message::RequestNodeRuntimeIP(node_id)).await {
                Ok(_) => tracing::debug!("[DEBUG] Successfully sent RequestNodeRuntimeIP"),
                Err(e) => tracing::debug!("[ERROR] Failed to send RequestNodeRuntimeIP: {:?}", e),
            }
        }

        if start.elapsed() > timeout {
            return Err("Timed out waiting for runtime IP".into());
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

pub async fn handle_peer_connection_spawn(
    executor: Arc<Mutex<StandardExecutor>>,
    node_id_map: Arc<Mutex<HashMap<NodeId, String>>>,
    abstract_flow: Arc<Mutex<AbstractFlow>>,
    execution_config: ExecutionConfig,
    orch_sender: Arc<Mutex<NetworkCommunicator<String>>>,
    orchestrator_ip: String,
    sending_node_id: NodeId,
    receiving_node_id: NodeId,
    sender_out_idx: NodeIOIndex,
    receiver_in_idx: NodeIOIndex,
    type_name: String,
    next_port: Arc<Mutex<u16>>,
) {
    if let Err(e) = handle_request_peer_connection_static(
        executor,
        node_id_map,
        abstract_flow,
        execution_config,
        orch_sender,
        orchestrator_ip,
        sending_node_id,
        receiving_node_id,
        sender_out_idx,
        receiver_in_idx,
        type_name,
        next_port,
    )
    .await
    {
        eprintln!("[Node RT] [R2R] Error: {}", e);
    }
}
