use crate::runtime::runtime_args::Arguments;
use anyhow::Error;
use flowrs::comm::communication::Communicator;
use flowrs::comm::messages::Message;
use flowrs::comm::network_communicator::NetworkCommunicator;
use flowrs::exec::execution_configuration::{ExecutionConfig, NodeConfig};
use flowrs::flow::abstract_flow::AbstractFlow;
use flowrs::flow::flow_types::NodeId;
use flowrs::sched::scheduling_config::{self, RuntimeId, SchedulingConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use tokio::task;
use tokio::time::sleep;

use crate::runtime::runtime_constants::{RUNTIME_PORT, SETUP_PORT};

pub struct Orchestrator {
    listener: TcpListener,
    connected_runtimes:
        Arc<Mutex<HashMap<String, (NetworkCommunicator, NetworkCommunicator, u16)>>>,
    runtime_id_map: Arc<Mutex<HashMap<u128, String>>>,
    next_port: Arc<Mutex<u16>>,
    abstract_flow: AbstractFlow,
    execution_config: ExecutionConfig,
}

impl Orchestrator {
    pub async fn new(
        abstract_flow: AbstractFlow,
        execution_config: ExecutionConfig,
    ) -> Result<Self, anyhow::Error> {
        // Bind the orchestrator's listener to the specified port
        let listener = TcpListener::bind(format!("0.0.0.0:{}", SETUP_PORT)).await?;

        // Initialize shared structures
        let connected_runtimes = Arc::new(Mutex::new(HashMap::new()));
        let runtime_id_map = Arc::new(Mutex::new(HashMap::new()));
        let next_port = Arc::new(Mutex::new(SETUP_PORT + 1)); // Start assigning ports from the next available port

        println!(
            "[Orchestrator] Listening for node runtimes on port {}...",
            SETUP_PORT
        );

        Ok(Self {
            listener,
            connected_runtimes,
            runtime_id_map,
            next_port,
            abstract_flow,
            execution_config,
        })
    }
    pub async fn run(
        self: Arc<Self>,
        args: Arguments,
        scheduling_config: SchedulingConfig,
    ) -> Result<(), Error> {
        println!(
            "[Orchestrator] Running as orchestrator with flow file: {}",
            args.flow
        );

        // =======================================================================================
        // Step 1: Connect to node runtimes
        // =======================================================================================
        let number_of_runtimes_expected = scheduling_config.runtime_nodes.len();
        // start listening for incoming runtime connections
        //let listener = TcpListener::bind(format!("0.0.0.0:{}", SETUP_PORT).as_str()).await?;
        println!(
            "[Orchestrator] Listening for node runtimes on port {}...",
            SETUP_PORT
        );

        // Store discovered runtimes (RuntimeId -> IP)
        let mut runtime_map: HashMap<RuntimeId, String> = HashMap::new();
        // Store connection to runtimes (IP -> (Sender, Receiver, sending_port)
        // let connected_runtimes = Arc::new(Mutex::new(HashMap::<
        //     String,
        //     (NetworkCommunicator, NetworkCommunicator, u16),
        // >::new()));
        let next_port = Arc::new(Mutex::new(RUNTIME_PORT + 1)); // start port assignments from the first free port

        // loop until number of runtimes matches
        while self.connected_runtimes.lock().await.len() < number_of_runtimes_expected {
            match self.listener.accept().await {
                Ok((mut stream, addr)) => {
                    let runtime_ip = addr.ip().to_string();

                    // Extract runtime ID from the stream
                    let mut buffer = [0; 32]; // Adjust buffer size if needed
                    if let Ok(size) = stream.read(&mut buffer).await {
                        let received_msg = String::from_utf8_lossy(&buffer[..size]).to_string();
                        if let Some(runtime_id_str) = received_msg.strip_prefix("RUNTIME_ID:") {
                            if let Ok(runtime_id) = runtime_id_str.trim().parse::<RuntimeId>() {
                                println!(
                                "[Orchestrator] Node runtime connected from {} (Runtime ID: {})...",
                                runtime_ip, runtime_id
                            );

                                // Store the mapping (Runtime ID -> IP)
                                runtime_map.insert(runtime_id, runtime_ip.clone());

                                // Check if this runtime is already known
                                let is_known_runtime = self
                                    .connected_runtimes
                                    .lock()
                                    .await
                                    .contains_key(&runtime_ip);
                                let connected_count = self.connected_runtimes.lock().await.len();

                                // If we have all expected runtimes AND it's NOT a reconnection, ignore the connection
                                if connected_count >= number_of_runtimes_expected
                                    && !is_known_runtime
                                {
                                    println!(
                                "[Orchestrator] Ignoring new connection from {}. Already at max runtimes ({}/{})",
                                runtime_ip, connected_count, number_of_runtimes_expected
                            );
                                    continue; // Ignore this connection
                                }

                                println!(
                                "[Orchestrator] Node runtime connected from {} (Runtime ID: {})...",
                                runtime_ip, runtime_id
                            );

                                let mut port_lock = next_port.lock().await;
                                let assigned_port = *port_lock;
                                *port_lock += 1;
                                drop(port_lock);
                                //let mut port_should_increment = true;

                                // Clone Arc so each task gets independent access
                                let connected_runtimes_clone = Arc::clone(&self.connected_runtimes);
                                let orchestrator = Arc::clone(&self);
                                let handle_task = task::spawn(async move {
                                    if let Err(e) = orchestrator
                                        .handle_runtime_connection(
                                            runtime_id,
                                            runtime_ip.clone(),
                                            connected_runtimes_clone,
                                            assigned_port,
                                        )
                                        .await
                                    {
                                        println!(
                                            "[Orchestrator] Error handling runtime {}: {}",
                                            runtime_ip, e
                                        );
                                    }
                                });

                                // Wait for the task to update `connected_runtimes`
                                let _ = handle_task.await;

                                // Re-check if all expected runtimes are connected
                                if self.connected_runtimes.lock().await.len()
                                    >= number_of_runtimes_expected
                                {
                                    break;
                                }
                            } else {
                                println!(
                                "[Orchestrator] ERROR: Invalid Runtime ID received from {}. Ignoring...",
                                runtime_ip
                            );
                            }
                        } else {
                            println!(
                                "[Orchestrator] ERROR: Malformed message from {}. Ignoring...",
                                runtime_ip
                            );
                        }
                    } else {
                        println!(
                            "[Orchestrator] ERROR: Failed to read runtime ID from {}",
                            runtime_ip
                        );
                    }
                }
                Err(e) => {
                    println!("[Orchestrator] Failed to accept connection: {}", e);
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
        println!("[Orchestrator] All expected runtimes connected!");

        // =======================================================================================
        // Step 6: Command Runtimes to Initialize Local Nodes
        // =======================================================================================

        println!("[Orchestrator] Sending node initialization requests sequentially...");

        for (runtime_ip, (sender, receiver, port)) in
            self.connected_runtimes.lock().await.iter_mut()
        {
            println!(
                "[DEBUG] Sending InitializeLocalNodes to {} on port {}",
                runtime_ip, port
            );

            // Send the initialization message
            let message = Message::<String>::InitializeLocalNodes;
            if let Err(e) = sender.send(message).await {
                println!(
                    "[Orchestrator] ERROR: Failed to send InitializeLocalNodes to {}: {}",
                    runtime_ip, e
                );
                continue; // Skip to next runtime
            }
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Wait for acknowledgment before moving to the next runtime
            loop {
                match Communicator::<String>::receive(receiver).await {
                    Ok(Message::AcknowledgeNodeInitialization) => {
                        println!(
                            "[Orchestrator] Received AcknowledgeNodeInitialization from {}",
                            runtime_ip
                        );
                        break; // Proceed to next runtime
                    }
                    Ok(Message::RequestNodeRuntimeIP(node_id)) => {
                        println!("[Orchestrator] Received IP request for Node {}", node_id);
                        self.handle_ip_request(sender, node_id).await;
                    }
                    Ok(msg) => {
                        println!(
                            "[Orchestrator] WARNING: Unexpected message from {}: {:?}",
                            runtime_ip, msg
                        );
                    }
                    Err(_) => {
                        println!(
                            "[Orchestrator] ERROR: Failed to receive acknowledgment from {}",
                            runtime_ip
                        );
                    }
                }
            }
        }
        println!("[Orchestrator] All runtimes have initialized their local nodes!");

        // =======================================================================================
        // Step 7: Setup Peer-to-Peer Connections Between Nodes
        // =======================================================================================
        let orchestrator_clone = Arc::clone(&self);
        println!("[Orchestrator] Initiating P2P connections...");
        if let Err(e) = orchestrator_clone.setup_p2p_connections().await {
            println!(
                "[Orchestrator] ERROR: Failed to setup P2P connections: {}",
                e
            );
        } else {
            println!("[Orchestrator] P2P connections established successfully.");
        }

        // =======================================================================================
        // Step 8: Start Execution
        // =======================================================================================
        println!("[Orchestrator] Starting execution...");

        for (runtime_ip, (sender, _, _)) in self.connected_runtimes.lock().await.iter_mut() {
            let start_msg = Message::<String>::StartExecution;

            if let Err(e) = sender.send(start_msg).await {
                println!(
                    "[Orchestrator] ERROR: Failed to send StartExecution to {}: {}",
                    runtime_ip, e
                );
            } else {
                println!("[Orchestrator] Sent StartExecution to {}", runtime_ip);
            }
        }
        // Keep the orchestrator running indefinitely
        loop {
            sleep(Duration::from_secs(60)).await;
        }
        Ok(())
    }

    async fn handle_runtime_connection(
        &self,
        runtime_id: RuntimeId,
        runtime_ip: String,
        connected_runtimes: Arc<
            Mutex<HashMap<String, (NetworkCommunicator, NetworkCommunicator, u16)>>,
        >,
        assigned_port: u16,
    ) -> Result<(), Error> {
        let mut port = assigned_port;
        let mut lock = connected_runtimes.lock().await;
        if lock.contains_key(&runtime_ip) {
            println!(
                "[Orchestrator] {} is already connected. Skipping reconnection.",
                runtime_ip
            );
            return Ok(());
        }
        if let Some((old_sender, old_receiver, old_port)) = lock.remove(&runtime_ip) {
            println!(
                "[Orchestrator] Detected reconnection from {}. Cleaning up old connections...",
                runtime_ip
            );
            drop(old_sender);
            drop(old_receiver);
            port = old_port;

            // Ensure socket is released
            sleep(Duration::from_millis(500)).await;
        }
        drop(lock); // Unlock mutex early to allow parallel connections

        // **Oneshot channel to signal receiver readiness**
        let (tx, rx) = oneshot::channel::<Result<NetworkCommunicator, Error>>();

        // =======================================================================================
        // Step 2: Spawn receiver task
        // =======================================================================================
        // **Spawn Receiver First**
        let receiver_runtime_ip = runtime_ip.clone();
        task::spawn(async move {
            let mut receiver = NetworkCommunicator::new().await.expect("should construct");

            match Communicator::<String>::connect_recv(
                &mut receiver,
                Some(receiver_runtime_ip.clone()),
                Some(port),
            )
            .await
            {
                Ok(_) => {
                    println!(
                        "[Orchestrator] Receiver bound to {}:{}",
                        receiver_runtime_ip, port
                    );

                    match <NetworkCommunicator as Communicator<String>>::receive::<'_, '_>(
                        &mut receiver,
                    )
                    .await
                    {
                        Ok(Message::AcknowledgeConnection) => {
                            println!(
                            "[Orchestrator] Acknowledgment received from runtime {} on port {}!",
                            receiver_runtime_ip, port
                        );
                            let _ = tx.send(Ok(receiver));
                        }
                        Ok(msg) => {
                            println!(
                                "[Orchestrator] [WARNING] Unexpected message from runtime {}: {:?}",
                                receiver_runtime_ip, msg
                            );
                            let _ = tx.send(Err(anyhow::Error::msg(
                                "Unexpected message instead of acknowledgment",
                            )));
                        }
                        Err(e) => {
                            println!(
                                "[Orchestrator] Failed to receive acknowledgment from {}: {}",
                                receiver_runtime_ip, e
                            );
                            let _ = tx.send(Err(anyhow::Error::msg(e.to_string())));
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "[Orchestrator] Receiver binding failed for {}:{} - {}",
                        receiver_runtime_ip, port, e
                    );
                    let _ = tx.send(Err(anyhow::Error::msg(e.to_string())));
                }
            }
        });

        sleep(Duration::from_millis(100)).await; // Ensure receiver is ready

        // =======================================================================================
        // Step 3: Create Sender
        // =======================================================================================
        // **Create Sender**
        let mut sender = NetworkCommunicator::new().await.expect("should construct");
        let mut retries = 0;
        while retries < 5 {
            match Communicator::<String>::connect_send(
                &mut sender,
                Some(runtime_ip.clone()),
                Some(RUNTIME_PORT),
            )
            .await
            {
                Ok(_) => {
                    println!(
                        "[Orchestrator] Sender connected to {}:{}",
                        runtime_ip, RUNTIME_PORT
                    );
                    break;
                }
                Err(e) => {
                    println!(
                        "[Orchestrator] Sender connection attempt {}/5 failed: {}",
                        retries + 1,
                        e
                    );
                    retries += 1;
                    // Increase wait time between retries
                    let wait_time = retries * 1000; // 1s, 2s, 3s, etc.
                    sleep(Duration::from_millis(wait_time as u64)).await;
                }
            }
        }
        // =======================================================================================
        // Step 4: Assign (receiving) Port to Runtime
        // =======================================================================================
        println!(
            "[Orchestrator] Assigning port {} to runtime {}...",
            port, runtime_ip
        );

        let setup_msg = Message::<String>::SetupCommunicationPort(assigned_port);
        println!("[Orchestrator] Sending message: {:?}", setup_msg);
        if let Err(e) = sender.send(setup_msg).await {
            println!(
                "[Orchestrator] Failed to send port assignment message to {}: {}",
                runtime_ip, e
            );
            return Ok(());
        }

        println!(
            "[Orchestrator] Sent SetupCommunicationPort message to {} for port {}",
            runtime_ip, port
        );
        //sleep(Duration::from_millis(100)).await;

        // Store temporary receiver entry to avoid missing receiver issue
        {
            let mut lock = connected_runtimes.lock().await;
            lock.insert(
                runtime_ip.clone(),
                (
                    <NetworkCommunicator as Communicator<String>>::clone_send(&sender),
                    NetworkCommunicator::new().await.unwrap(),
                    port,
                ),
            );
            drop(lock); // Unlock before waiting for acknowledgment
        }

        // Wait for Receiver to be Ready
        let receiver = match rx.await {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => {
                println!("[Orchestrator] Receiver failed: {}", e);
                return Ok(());
            }
            Err(_) => {
                println!("[Orchestrator] Receiver channel closed unexpectedly");
                return Ok(());
            }
        };

        // =======================================================================================
        // Step 5: Store Sender and Receiver
        // =======================================================================================
        // Store the communicator in the HashMap
        let mut lock = connected_runtimes.lock().await;
        lock.insert(runtime_ip.clone(), (sender, receiver, port));

        let mut id_map = self.runtime_id_map.lock().await;
        id_map.insert(runtime_id, runtime_ip.clone()); // Store the mapping
        drop(id_map);

        println!(
            "[Orchestrator] Successfully registered runtime {} with port {}. [{} Total]",
            runtime_ip,
            port,
            lock.len()
        );

        Ok(())
    }

    /// Initiates peer-to-peer node connections one at a time, awaiting confirmation before proceeding.
    /// Initiates peer-to-peer node connections one at a time, awaiting confirmation before proceeding.
    /// Initiates peer-to-peer node connections one at a time, awaiting confirmation before proceeding.
    pub async fn setup_p2p_connections(self: Arc<Self>) -> Result<(), anyhow::Error> {
        println!("[Orchestrator] Starting P2P node connections...");

        let connections = self
            .abstract_flow
            .get_connections()
            .cloned()
            .collect::<Vec<_>>();

        for connection in connections {
            let sender_id = connection.sender_id;
            let receiver_id = connection.receiver_id;
            let sender_out_idx = connection.send_out_idx;
            let recv_in_idx = connection.recv_in_idx;

            let sender_runtime = self.execution_config.node_configs.get(&sender_id);
            let receiver_runtime = self.execution_config.node_configs.get(&receiver_id);

            println!(
            "[DEBUG] Checking connection: sender_id={:?} (runtime={:?}), receiver_id={:?} (runtime={:?})",
            sender_id, sender_runtime, receiver_id, receiver_runtime
        );

            // ✅ Fix: Ensure `get_runtime_ip()` returns a `String` instead of a Future
            let sender_ip = self.get_runtime_ip(sender_id).await;

            println!(
                "[DEBUG] Checking if sender_ip={} is registered in connected_runtimes...",
                sender_ip
            );

            let mut connected_runtimes_guard = self.connected_runtimes.lock().await;
            if let Some((sender_comm, receiver_comm, port)) =
                connected_runtimes_guard.get_mut(&sender_ip)
            {
                println!(
                    "[Orchestrator] Maintaining sender and receiver connections for {}:{}",
                    sender_ip, port
                );
                println!(
                    "[Orchestrator] Requesting P2P connection from Node {} to Node {}",
                    sender_id, receiver_id
                );

                let runtime_id = self.execution_config.runtime_id;
                let runtime_ip = sender_ip.clone();

                let message = Message::<String>::OrchestratorRequestNodeConnection(
                    sender_id,
                    receiver_id,
                    runtime_id,
                    runtime_ip,
                    sender_out_idx,
                    recv_in_idx,
                );

                if let Err(e) = sender_comm.send(message).await {
                    println!(
                        "[Orchestrator] ERROR: Failed to request P2P connection: {}",
                        e
                    );
                    return Err(anyhow::Error::msg(e.to_string()));
                }

                // Wait for acknowledgment before proceeding
                match self.await_connection_ack(sender_id).await {
                    Ok(_) => println!(
                        "[Orchestrator] Connection {} -> {} established successfully!",
                        sender_id, receiver_id
                    ),
                    Err(e) => println!(
                        "[Orchestrator] ERROR: Failed to establish connection {} -> {}: {}",
                        sender_id, receiver_id, e
                    ),
                }
            } else {
                println!(
                    "[Orchestrator] ERROR: No sender communication channel found for sender_ip={}",
                    sender_ip
                );
            }
        }

        println!("[Orchestrator] All P2P connections processed.");
        Ok(())
    }

    /// Awaits acknowledgment from the sender node before continuing with the next connection.
    async fn await_connection_ack(&self, sender_id: NodeId) -> Result<(), anyhow::Error> {
        let sender_ip = self.get_runtime_ip(sender_id).await;
        if let Some((_, receiver_comm, _)) =
            self.connected_runtimes.lock().await.get_mut(&sender_ip)
        {
            match <NetworkCommunicator as Communicator<String>>::receive::<'_, '_>(receiver_comm)
                .await
            {
                Ok(Message::AcknowledgeConnectionSetup(..)) => Ok(()),
                Ok(_) => Err(anyhow::Error::msg(
                    "Unexpected message received while waiting for acknowledgment",
                )),
                Err(e) => Err(anyhow::Error::msg(format!(
                    "Failed to receive acknowledgment: {}",
                    e
                ))),
            }
        } else {
            Err(anyhow::Error::msg("Sender runtime not found"))
        }
    }

    async fn get_runtime_ip(&self, node_id: NodeId) -> String {
        if let Some(NodeConfig::RemoteNodeConfig(runtime_id)) =
            self.execution_config.node_configs.get(&node_id)
        {
            let id_map = self.runtime_id_map.lock().await;
            if let Some(ip) = id_map.get(runtime_id) {
                println!("[DEBUG] Found IP {} for runtime_id={}", ip, runtime_id);
                return ip.clone();
            }
        }

        println!(
            "[Orchestrator] ERROR: Could not find runtime IP for node {}",
            node_id
        );
        "UNKNOWN_IP".to_string()
    }

    async fn handle_ip_request(&self, sender: &mut NetworkCommunicator, node_id: NodeId) {
        println!("[Orchestrator] Received IP request for Node {}", node_id);

        // Find the runtime that owns this node
        let runtime_ip = self.get_runtime_ip(node_id).await;

        if runtime_ip == "UNKNOWN_IP" {
            println!(
                "[Orchestrator] ERROR: Cannot find runtime IP for node {}",
                node_id
            );
            return;
        }

        // Send back the IP address to the requesting runtime
        let response = Message::<String>::RespondNodeRuntimeIP(node_id, runtime_ip.clone());

        if let Err(e) = sender.send(response).await {
            println!(
                "[Orchestrator] ERROR: Failed to send IP response to requesting runtime: {}",
                e
            );
        } else {
            println!(
                "[Orchestrator] Sent IP response for Node {}: IP={}",
                node_id, runtime_ip
            );
        }
    }
}
