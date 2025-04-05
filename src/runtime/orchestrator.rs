use crate::runtime::runtime_args::Arguments;
use crate::runtime::runtime_constants::{RUNTIME_PORT, SETUP_PORT};
use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use flowrs::comm::communication::Communicator;
use flowrs::comm::messages::Message;
use flowrs::comm::network_communicator::NetworkCommunicator;
use flowrs::exec::execution_configuration::ExecutionConfig;
use flowrs::exec::execution_configuration::NodeConfig;
use flowrs::flow::abstract_flow::AbstractFlow;
use flowrs::flow::flow_types::{NodeIOIndex, NodeId};
use flowrs::sched::scheduling_config::{RuntimeId, SchedulingConfig};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{oneshot, Mutex};
use tokio::time::{sleep, Instant};
use tokio::{spawn, task};
pub struct Orchestrator {
    //_listener: TcpListener,
    connected_runtimes: Arc<
        Mutex<
            HashMap<
                String,
                (
                    NetworkCommunicator<String>,
                    //NetworkCommunicator<String>,
                    u16,
                ),
            >,
        >,
    >,
    runtime_id_map: Arc<Mutex<HashMap<u128, String>>>,
    next_port: Arc<Mutex<u16>>,
    abstract_flow: AbstractFlow,
    execution_config: ExecutionConfig,
    // orch_receiver: Arc<Mutex<NetworkCommunicator<String>>>,
    orch_channel_tx: Sender<(RuntimeId, Message<String>)>,
    orch_channel_rx: Arc<Mutex<Receiver<(RuntimeId, Message<String>)>>>,
    pending_connections: Arc<Mutex<HashMap<NodeId, (NodeId, NodeIOIndex, NodeIOIndex)>>>,
}

impl Orchestrator {
    pub async fn new(
        abstract_flow: AbstractFlow,
        execution_config: ExecutionConfig,
    ) -> Result<Self, anyhow::Error> {
        // Bind the orchestrator's listener to the specified port
        //let listener = TcpListener::bind(format!("0.0.0.0:{}", SETUP_PORT)).await?;

        // Initialize shared structures
        let connected_runtimes = Arc::new(Mutex::new(HashMap::new()));
        let runtime_id_map = Arc::new(Mutex::new(HashMap::new()));
        let _next_port = Arc::new(Mutex::new(SETUP_PORT + 1)); // Start assigning ports from the next available port
        let (orch_channel_tx, orch_channel_rx) = channel::<(RuntimeId, Message<String>)>(64);
        let pending_connections = Arc::new(Mutex::new(HashMap::new()));
        let orch_rx_arc = Arc::new(Mutex::new(orch_channel_rx));
        let next_port = Arc::new(Mutex::new(RUNTIME_PORT + 1));

        println!(
            "[Orchestrator] Listening for node runtimes on port {}...",
            SETUP_PORT
        );

        Ok(Self {
            //_listener,
            connected_runtimes,
            runtime_id_map,
            next_port,
            abstract_flow,
            execution_config,
            orch_channel_tx,
            orch_channel_rx: orch_rx_arc,
            pending_connections,
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

        let number_of_runtimes_expected = scheduling_config.runtime_nodes.len();

        let _runtime_map: HashMap<RuntimeId, String> = HashMap::new();

        // === Step 1: Accept all runtime connections and assign ports ===
        let self_clone = Arc::clone(&self);
        self_clone
            .accept_runtime_connections(number_of_runtimes_expected)
            .await?;

        println!("[Orchestrator] All expected runtimes connected!");

        // === Step 2: Initialize all local nodes across runtimes ===
        self.initialize_remote_nodes().await?;

        // === Step 3: Wait for AcknowledgeNodeInitialization from each runtime ===
        self.wait_for_node_initialization().await?;

        println!("[Orchestrator] All runtimes have initialized their local nodes!");

        // === Step 4: Initiate peer-to-peer node connections ===
        println!("[Orchestrator] Initiating P2P connections...");
        self.clone().setup_p2p_connections().await?;

        // === Step 5: Wait for AcknowledgeConnectionSetup messages ===
        println!("[Orchestrator] Waiting for P2P-Connection Acknowledge from all runtimes...");
        self.wait_for_connection_acknowledgments().await?;

        println!("[Orchestrator] All P2P connections processed.");

        // === Step 6: Start distributed execution ===
        println!("[Orchestrator] P2P connections established successfully.");
        println!("[Orchestrator] Starting execution...");
        self.send_start_execution()
            .await
            .map_err(|e| anyhow::Error::msg(format!("Failed to start execution {}", e)))?;
        // Keep the orchestrator running indefinitely
        loop {
            sleep(Duration::from_secs(60)).await;
        }

        #[allow(unreachable_code)]
        Ok(())
    }

    /// Sets up peer-to-peer connections between nodes based on the abstract flow.
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

            let sender_ip = self.get_runtime_ip(sender_id).await;
            let receiver_ip = self.get_runtime_ip(receiver_id).await;

            println!(
                "[DEBUG] Checking if sender_ip={} is registered in connected_runtimes...",
                sender_ip
            );

            let mut connected_runtimes_guard = self.connected_runtimes.lock().await;
            if let Some((sender_comm, port)) = connected_runtimes_guard.get_mut(&sender_ip) {
                println!(
                "[Orchestrator] Successfully found communication channels for sender_ip={} (port={})",
                sender_ip, port
            );

                println!(
                    "[Orchestrator] Requesting P2P connection from Node {} on {} to Node {} on {}",
                    sender_id, sender_ip, receiver_id, receiver_ip
                );

                let runtime_id = match receiver_runtime {
                    Some(NodeConfig::RemoteNodeConfig(id)) => *id,
                    _ => {
                        return Err(anyhow::Error::msg(format!(
                            "Receiver runtime ID not found for node {}",
                            receiver_id
                        )));
                    }
                };
                let runtime_ip = receiver_ip.clone();

                let message = Message::<String>::OrchestratorRequestNodeConnection(
                    sender_id,
                    receiver_id,
                    runtime_id,
                    runtime_ip,
                    sender_out_idx,
                    recv_in_idx,
                );

                // Send the P2P connection request
                if let Err(e) = sender_comm.send(message).await {
                    println!(
                        "[Orchestrator] ERROR: Failed to request P2P connection: {}",
                        e
                    );
                    return Err(anyhow::Error::msg(e.to_string()));
                }

                // Insert into the pending connection tracker instead of blocking
                self.pending_connections
                    .lock()
                    .await
                    .insert(sender_id, (receiver_id, sender_out_idx, recv_in_idx));

                println!(
                "[Orchestrator] P2P connection request sent for {} -> {}. Awaiting acknowledgment later.",
                sender_id, receiver_id
            );
            } else {
                println!(
                    "[Orchestrator] ERROR: No sender communication channel found for sender_ip={}",
                    sender_ip
                );
                return Err(anyhow::Error::msg("No sender communication channel found"));
            }
        }

        println!("[Orchestrator] All P2P connection requests sent.");
        Ok(())
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

    pub async fn accept_runtime_connections(
        self: Arc<Self>,
        number_of_runtimes_expected: usize,
    ) -> Result<(), Error> {
        // =======================================================================================
        // Step 1: Connect to node runtimes
        // =======================================================================================
        //start listening for incoming runtime connections
        let listener = TcpListener::bind(format!("0.0.0.0:{}", SETUP_PORT).as_str()).await?;
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

        //let listener = TcpListener::bind(format!("0.0.0.0:{}", SETUP_PORT)).await?;
        // loop until number of runtimes matches
        while self.connected_runtimes.lock().await.len() < number_of_runtimes_expected {
            match listener.accept().await {
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
                                let orchestrator = Arc::clone(&self);
                                let handle_task = task::spawn(async move {
                                    if let Err(e) = orchestrator
                                        .handle_runtime_connection(
                                            runtime_id,
                                            runtime_ip.clone(),
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

        Ok(())
    }

    async fn handle_runtime_connection(
        &self,
        runtime_id: RuntimeId,
        runtime_ip: String,
        mut assigned_port: u16,
    ) -> Result<(), Error> {
        let mut lock = self.connected_runtimes.lock().await;
        if lock.contains_key(&runtime_ip) {
            println!(
                "[Orchestrator] {} is already connected. Skipping reconnection.",
                runtime_ip
            );
            return Ok(());
        }
        if let Some((old_sender, old_port)) = lock.remove(&runtime_ip) {
            println!(
                "[Orchestrator] Detected reconnection from {}. Cleaning up old connections...",
                runtime_ip
            );
            drop(old_sender);
            assigned_port = old_port;

            // Ensure socket is released
            sleep(Duration::from_millis(500)).await;
        }
        drop(lock); // Unlock mutex early to allow parallel connections

        // **Oneshot channel to signal receiver readiness**
        let (tx, rx) = oneshot::channel::<Result<NetworkCommunicator<String>, Error>>();

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
                Some(assigned_port),
            )
            .await
            {
                Ok(_) => {
                    println!(
                        "[Orchestrator] Receiver bound to {}:{}",
                        receiver_runtime_ip, assigned_port
                    );

                    match receiver.receive().await {
                        Ok(Message::AcknowledgeConnection) => {
                            println!(
                            "[Orchestrator] Acknowledgment received from runtime {} on port {}!",
                            receiver_runtime_ip, assigned_port
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
                        receiver_runtime_ip, assigned_port, e
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
            assigned_port, runtime_ip
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
            runtime_ip, assigned_port
        );

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
        let mut lock = self.connected_runtimes.lock().await;
        lock.insert(runtime_ip.clone(), (sender, assigned_port));

        let mut id_map = self.runtime_id_map.lock().await;
        id_map.insert(runtime_id, runtime_ip.clone()); // Store the mapping
        drop(id_map);

        println!(
            "[Orchestrator] Successfully registered runtime {} with port {}. [{} Total]",
            runtime_ip,
            assigned_port,
            lock.len()
        );

        // Spawn the runtime receiving loop
        self.spawn_runtime_loop(receiver, runtime_id.clone(), self.orch_channel_tx.clone());

        Ok(())
    }

    pub async fn initialize_remote_nodes(&self) -> Result<(), Error> {
        let mut connected_runtimes = self.connected_runtimes.lock().await;

        for (runtime_ip, (sender, assigned_port)) in connected_runtimes.iter_mut() {
            println!(
                "[Orchestrator] Sending node initialization to {} on port {}...",
                runtime_ip, assigned_port
            );

            sender
                .send(Message::InitializeLocalNodes)
                .await
                .map_err(|e| anyhow::Error::msg(format!("Failed to send config message {}", e)))?;

            println!(
                "[DEBUG] Sent InitializeLocalNodes to {} on port {}",
                runtime_ip, assigned_port
            );
        }

        println!("[Orchestrator] Sent node initialization requests sequentially.");
        Ok(())
    }

    pub fn spawn_runtime_loop(
        &self,
        mut receiver: NetworkCommunicator<String>,
        runtime_id: RuntimeId,
        orch_channel_tx: Sender<(RuntimeId, Message<String>)>,
    ) {
        tokio::spawn(async move {
            loop {
                match receiver.try_receive().await {
                    Ok(Some(msg)) => {
                        println!(
                            "[Orchestrator] [recv-loop] Message from runtime {:?}: {:?}",
                            runtime_id, msg
                        );
                        if let Err(e) = orch_channel_tx.send((runtime_id, msg)).await {
                            println!(
                            "[Orchestrator] [recv-loop] Failed to forward message from {:?}: {}",
                            runtime_id, e
                        );
                            break;
                        }
                    }
                    Ok(None) => {
                        // No message available — yield to other tasks briefly
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                    Err(e) => {
                        println!(
                            "[Orchestrator] [recv-loop] Error receiving from runtime {:?}: {}",
                            runtime_id, e
                        );
                        break;
                    }
                }
            }
        });
    }

    pub async fn wait_for_node_initialization(&self) -> Result<(), Error> {
        let expected_runtimes: usize = self
            .execution_config
            .node_configs
            .values()
            .filter_map(|cfg| match cfg {
                NodeConfig::RemoteNodeConfig(runtime_id) => Some(*runtime_id),
                NodeConfig::LocalNodeConfig => None,
            })
            .collect::<std::collections::HashSet<_>>()
            .len();

        println!(
            "[Orchestrator] Waiting for AcknowledgeNodeInitialization from {} runtimes...",
            expected_runtimes
        );

        let mut received_acks = 0;

        while received_acks < expected_runtimes {
            let mut rx = self.orch_channel_rx.lock().await;

            match rx.try_recv() {
                Ok((_runtime_id, Message::AcknowledgeNodeInitialization)) => {
                    received_acks += 1;
                    println!(
                        "[Orchestrator] Received AcknowledgeNodeInitialization ({}/{})",
                        received_acks, expected_runtimes
                    );
                }
                Ok(msg) => {
                    println!(
                        "[Orchestrator] Unexpected message while waiting for node init: {:?}",
                        msg
                    );
                }
                Err(TryRecvError::Empty) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
                Err(TryRecvError::Disconnected) => {
                    return Err(anyhow::anyhow!(
                        "orch_channel_rx disconnected while waiting for node initialization"
                    ));
                }
            }
        }

        Ok(())
    }

    // async fn start_p2p_node_connections(&self) -> Result<(), Error> {
    //     let flow = &self.abstract_flow;
    //     let _expected_connections = flow.get_connection_amount();
    //     println!("[Orchestrator] Starting P2P node connections...");

    //     for conn in flow.get_connections() {
    //         println!(
    //         "[DEBUG] Checking connection: sender_id={} (runtime={:?}), receiver_id={} (runtime={:?})",
    //         conn.sender_id,
    //         self.execution_config.node_configs.get(&conn.sender_id),
    //         conn.receiver_id,
    //         self.execution_config.node_configs.get(&conn.receiver_id)
    //     );

    //         let sender_runtime = self.execution_config.node_configs.get(&conn.sender_id);
    //         let receiver_runtime = self.execution_config.node_configs.get(&conn.receiver_id);

    //         match (sender_runtime, receiver_runtime) {
    //             (
    //                 Some(NodeConfig::RemoteNodeConfig(runtime_id)),
    //                 Some(NodeConfig::RemoteNodeConfig(_)),
    //             ) => {
    //                 let ip_str = self.get_runtime_ip(*runtime_id).await;
    //                 let connected = self.connected_runtimes.lock().await;
    //                 if let Some((_, port)) = connected.get(&ip_str) {
    //                     // Now you can use `port`
    //                     println!(
    //                         "[Orchestrator] Requesting P2P connection from Node {} on {} to Node {} on {}",
    //                         conn.sender_id,
    //                         ip_str,
    //                         conn.receiver_id,
    //                         ip_str,
    //                     );

    //                     let msg = Message::OrchestratorRequestNodeConnection(
    //                         conn.sender_id,
    //                         conn.receiver_id,
    //                         *runtime_id,
    //                         ip_str.clone(), // Clone if needed again later
    //                         conn.send_out_idx,
    //                         conn.recv_in_idx,
    //                     );
    //                     let mut sender = NetworkCommunicator::<String>::new()
    //                         .await
    //                         .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    //                     Communicator::connect_send(&mut sender, Some(ip_str.clone()), Some(*port))
    //                         .await
    //                         .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    //                     sender
    //                         .send(msg)
    //                         .await
    //                         .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    //                 }
    //             }
    //             _ => {
    //                 println!("[DEBUG] Skipping local or invalid connection {:?}", conn);
    //             }
    //         }
    //     }

    //     Ok(())
    // }

    pub async fn wait_for_connection_acknowledgments(&self) -> Result<(), Error> {
        let expected_acks = self.abstract_flow.get_connection_amount();

        println!(
            "[Orchestrator] Waiting for AcknowledgeConnectionSetup from {} connections...",
            expected_acks
        );

        let mut received_acks = HashSet::new();
        let start_time = Instant::now();
        let timeout = Duration::from_secs(10);

        while received_acks.len() < expected_acks {
            if start_time.elapsed() > timeout {
                break;
            }

            let msg = {
                let mut rx = self.orch_channel_rx.lock().await;
                tokio::select! {
                    msg = rx.recv() => msg,
                    _ = sleep(Duration::from_millis(50)) => continue,
                }
            };

            match msg {
                Some((
                    runtime_id,
                    Message::AcknowledgeConnectionSetup(
                        sender_id,
                        receiver_id,
                        sender_out_idx,
                        receiver_in_idx,
                    ),
                )) => {
                    println!(
                    "[Orchestrator] Received AcknowledgeConnectionSetup from sender {} to receiver {} (out_idx={}, in_idx={}) (runtime_id={})",
                    sender_id, receiver_id, sender_out_idx, receiver_in_idx, runtime_id
                );
                    received_acks.insert((sender_id, receiver_id, sender_out_idx, receiver_in_idx));
                }
                Some(other) => {
                    println!(
                        "[Orchestrator] Unexpected message while waiting for connection acks: {:?}",
                        other
                    );
                }
                None => {
                    println!("[Orchestrator] Channel closed unexpectedly.");
                    break;
                }
            }
        }

        if received_acks.len() < expected_acks {
            return Err(anyhow!(
                "Timeout: Only received {} out of {} connection acknowledgments",
                received_acks.len(),
                expected_acks
            ));
        }

        Ok(())
    }

    async fn send_start_execution(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("[Orchestrator] Sending StartExecution command to all runtimes...");

        let mut connected = self.connected_runtimes.lock().await;

        // Immutable phase
        for (runtime_ip, (_, port)) in connected.iter() {
            println!("[...] Will send to {}:{}", runtime_ip, port);
        }

        // Mutable phase (in separate scope)
        for (_runtime_ip, (sender, _)) in connected.iter_mut() {
            let start_msg = Message::<String>::StartExecution;
            sender
                .send(start_msg)
                .await
                .map_err(|e| anyhow::Error::msg(e.to_string()))?;
        }

        Ok(())
    }

    // async fn handle_ip_request(&self, sender: &mut NetworkCommunicator<String>, node_id: NodeId) {
    //     println!("[Orchestrator] Received IP request for Node {}", node_id);

    //     // Find the runtime that owns this node
    //     let runtime_ip = self.get_runtime_ip(node_id).await;

    //     if runtime_ip == "UNKNOWN_IP" {
    //         println!(
    //             "[Orchestrator] ERROR: Cannot find runtime IP for node {}",
    //             node_id
    //         );
    //         return;
    //     }

    //     // Send back the IP address to the requesting runtime
    //     let response = Message::<String>::RespondNodeRuntimeIP(node_id, runtime_ip.clone());

    //     if let Err(e) = sender.send(response).await {
    //         println!(
    //             "[Orchestrator] ERROR: Failed to send IP response to requesting runtime: {}",
    //             e
    //         );
    //     } else {
    //         println!(
    //             "[Orchestrator] Sent IP response for Node {}: IP={}",
    //             node_id, runtime_ip
    //         );
    //     }
    // }
}
