use flowrs::comm::messages::Message;
use flowrs::exec::execution::StandardExecutor;
use flowrs::exec::execution_configuration::ExecutionConfig;
use flowrs::node::Node;
use flowrs::sched::scheduling_config;
use flowrs::sched::scheduling_config::RuntimeId;
use flowrs::sched::scheduling_config::SchedulingConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::lookup_host;

use anyhow::Error;
use clap::Parser;
use flowrs::comm::communication::Communicator;
use flowrs::comm::network_communicator::NetworkCommunicator;
use flowrs::flow::abstract_flow::AbstractFlow;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{sleep, timeout, Duration};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Shared library file of the flow to run.
    #[arg(short, long)]
    flow: String,

    /// Number of workers to use.
    #[arg(short, long, default_value_t = 1)]
    workers: usize,

    /// Role of the runtime (orchestrator or node-runtime).
    #[arg(short, long)]
    role: String,

    /// Runtime ID (only required for node runtimes)
    #[arg(long)]
    runtime_id: Option<usize>,
}

// #[derive(Clone)]
// struct ExecutionContextHandlePtr {
//     ptr: *mut ExecutionContextHandle,
// }
// unsafe impl Send for ExecutionContextHandlePtr {}

async fn get_orchestrator_address() -> Result<SocketAddr, anyhow::Error> {
    let mut addrs = lookup_host("orchestrator:5000").await?;
    addrs
        .next()
        .ok_or_else(|| anyhow::Error::msg("No valid IP found for orchestrator"))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Define the CLI application using clap
    let args = Arguments::parse();

    // Step 1: Create the flow definition
    let abstract_flow = return_dummy_flow()?;

    // Step 2: Generate the scheduling configuration (Global)
    let num_runtimes = 2; // Hardcoded for now, later can be dynamically set
    let scheduling_config = dummy_scheduling(&abstract_flow, num_runtimes);

    // Step 3: Get the orchestrator's address
    let orchestrator_addr = get_orchestrator_address().await?;

    println!(
        "Orchestrator Address: {}",
        orchestrator_addr.ip().to_string()
    );

    // Step 4: Determine role and start the appropriate component
    match args.role.as_str() {
        "orchestrator" => {
            run_orchestrator(args, abstract_flow, scheduling_config).await?;
        }
        "node-runtime" => {
            // Extract runtime ID from CLI arguments (needs to be added)
            let runtime_id: RuntimeId = args.runtime_id.expect("Missing runtime ID");

            // Generate execution config from the global scheduling config
            let execution_config =
                ExecutionConfig::from_scheduling_config(&scheduling_config, runtime_id);

            run_node_runtime(args, abstract_flow, orchestrator_addr, execution_config).await?;
        }
        _ => {
            eprintln!("Invalid role specified. Use 'orchestrator' or 'node-runtime'.");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn return_dummy_flow() -> Result<AbstractFlow, Error> {
    use flowrs_std::add::AddNode;
    use flowrs_std::value::ValueNode;

    //Define nodes
    let number_node_1: ValueNode<u32> = ValueNode::<u32>::new(3);
    let number_node_2: ValueNode<u32> = ValueNode::<u32>::new(2);
    let add_node = AddNode::<u32, u32, u32>::new();

    //Define flow
    let mut flow = AbstractFlow::new_empty();
    flow.add_node_with_id(Box::new(number_node_1), 1);
    flow.add_node_with_id(Box::new(number_node_2), 2);
    flow.add_node_with_id(Box::new(add_node), 3);
    flow.connect_nodes(1, 3, 0, 0)?;
    flow.connect_nodes(2, 3, 0, 1)?;

    Ok(flow)
}

fn dummy_scheduling(abstract_flow: &AbstractFlow, num_runtimes: usize) -> SchedulingConfig {
    let mut scheduling_config = SchedulingConfig::new();

    let mut runtime_id = 1;

    for node_id in abstract_flow.get_nodes().map(|(id, _)| id) {
        scheduling_config.assign_node(runtime_id, *node_id);
        runtime_id = ((runtime_id) % num_runtimes) + 1;
    }

    scheduling_config
}

async fn handle_runtime_connection(
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

    println!(
        "[Orchestrator] Successfully registered runtime {} with port {}. [{} Total]",
        runtime_ip,
        port,
        lock.len()
    );

    Ok(())
}

// Logic for running as orchestrator
async fn run_orchestrator(
    args: Arguments,
    abstract_flow: AbstractFlow,
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
    let listener = TcpListener::bind(format!("0.0.0.0:{}", SETUP_PORT).as_str()).await?;
    println!(
        "[Orchestrator] Listening for node runtimes on port {}...",
        SETUP_PORT
    );

    // Store discovered runtimes (RuntimeId -> IP)
    let mut runtime_map: HashMap<RuntimeId, String> = HashMap::new();
    // Store connection to runtimes (IP -> (Sender, Receiver, sending_port)
    let connected_runtimes = Arc::new(Mutex::new(HashMap::<
        String,
        (NetworkCommunicator, NetworkCommunicator, u16),
    >::new()));
    let next_port = Arc::new(Mutex::new(RUNTIME_PORT + 1)); // start port assignments from the first free port

    // loop until number of runtimes matches
    while connected_runtimes.lock().await.len() < number_of_runtimes_expected {
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
                            let is_known_runtime =
                                connected_runtimes.lock().await.contains_key(&runtime_ip);
                            let connected_count = connected_runtimes.lock().await.len();

                            // If we have all expected runtimes AND it's NOT a reconnection, ignore the connection
                            if connected_count >= number_of_runtimes_expected && !is_known_runtime {
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
                            let mut assigned_port = *port_lock;
                            *port_lock += 1;
                            drop(port_lock);
                            //let mut port_should_increment = true;

                            // Clone Arc so each task gets independent access
                            let connected_runtimes_clone = Arc::clone(&connected_runtimes);
                            let handle_task = task::spawn(async move {
                                if let Err(e) = handle_runtime_connection(
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
                            if connected_runtimes.lock().await.len() >= number_of_runtimes_expected
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

    for (runtime_ip, (sender, receiver, port)) in connected_runtimes.lock().await.iter_mut() {
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
    // Keep the orchestrator running indefinitely
    loop {
        sleep(Duration::from_secs(60)).await;
    }
    Ok(())
}

/// Logic for running as a node runtime
async fn run_node_runtime(
    args: Arguments,
    mut abstract_flow: AbstractFlow,
    orch_addr: SocketAddr,
    execution_config: ExecutionConfig,
) -> Result<(), Error> {
    println!(
        "[Node RT] Running as node runtime with flow file: {}",
        args.flow
    );

    let orchestrator_ip = orch_addr.ip().to_string();
    let mut assigned_port: u16 = 0;
    let runtime_id = execution_config.runtime_id;

    // Shared receiver reference to persist across retries
    let receiver_shared = Arc::new(Mutex::new(None));
    let receiver_shared_clone = Arc::clone(&receiver_shared);

    // =======================================================================================
    // Step 1: Create a receiver communicator to receive on RUNTIME_PORT
    // =======================================================================================
    let _recv_task = task::spawn(async move {
        let mut orch_receiver = NetworkCommunicator::new().await.expect("should construct");

        match <NetworkCommunicator as Communicator<String>>::connect_recv::<'_, '_>(
            &mut orch_receiver,
            Some(orch_addr.ip().to_string()),
            Some(RUNTIME_PORT),
        )
        .await
        {
            Ok(_) => {
                println!(
                    "[Node RT] Receiver successfully established on port {}",
                    RUNTIME_PORT
                );
                // Store the receiver in the shared reference
                let mut receiver_guard = receiver_shared_clone.lock().await;
                *receiver_guard = Some(orch_receiver);
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

        let mut receiver_guard = receiver_shared.lock().await;
        match timeout(recv_timeout, async {
            loop {
                if let Some(receiver) = &mut *receiver_guard {
                    return Communicator::<String>::receive(receiver).await;
                }
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
    let mut executor = StandardExecutor::new();
    let mut receiver_guard = receiver_shared.lock().await;

    if let Some(receiver) = &mut *receiver_guard {
        println!(
            "[Node RT] Waiting for InitializeLocalNodes on port {}",
            assigned_port
        );

        loop {
            match Communicator::<String>::receive(receiver).await {
                Ok(Message::InitializeLocalNodes) => {
                    println!("[Node RT] Received InitializeLocalNodes request...");

                    // ✅ Perform actual node initialization
                    if let Err(e) = executor
                        .initialize_nodes(&mut abstract_flow, &execution_config)
                        .await
                    {
                        println!("[Node RT] ERROR: Failed to initialize nodes: {}", e);
                        return Err(anyhow::Error::msg("Node initialization failed"));
                    }

                    println!("[Node RT] Successfully initialized local nodes.");

                    // ✅ Call `on_ready()` for all nodes after initialization
                    for node in executor.execution_nodes.values() {
                        let mut node_guard = node.lock().await;
                        if let Err(e) = node_guard.on_ready() {
                            println!("[Node RT] ERROR: Node failed to enter ready state: {}", e);
                            return Err(anyhow::Error::msg("Node ready state failed"));
                        }
                    }

                    println!("[Node RT] All nodes are ready!");

                    // ✅ Send acknowledgment back to the orchestrator
                    let ack_message = Message::<String>::AcknowledgeNodeInitialization;
                    if let Err(e) = orch_sender.send(ack_message).await {
                        println!(
                            "[Node RT] ERROR: Failed to send AcknowledgeNodeInitialization: {}",
                            e
                        );
                    } else {
                        println!("[Node RT] Sent AcknowledgeNodeInitialization.");
                    }
                    break; // Exit loop after successful initialization
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
    } else {
        println!("[Node RT] ERROR: Receiver was not initialized properly.");
        return Err(anyhow::Error::msg("Receiver not available"));
    }

    // Keep the runtime running indefinitely
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }

    // Now that communication is set up, node runtime is ready for execution.
    // TODO: Implement further logic for execution handling.

    // if communicator.stream.is_none() {
    //     println!("[Node RT] Could not establish connection to orchestrator. Exiting.");
    //     return Err(Error::ConnectionFailed);
    // }

    // Load the dynamic library
    //println!("-> Load flow from {}.", args.flow);

    // Step 1: Load flow data
    // Step 2: Start local nodes
    // Step 3: Await connection from orchestrator
    // Step 4: Run Nodes
    // Step 5: Clean up
    // unsafe {
    //     let lib = libloading::Library::new(args.flow).expect("Failed to load the dynamic library");

    //     let init_func: libloading::Symbol<unsafe extern "C" fn() -> *mut ExecutionContextHandle> =
    //         lib.get(b"native_init").expect("Not load.");
    //     let run_func: libloading::Symbol<
    //         unsafe extern "C" fn(usize, *mut ExecutionContextHandle) -> *const c_char,
    //     > = lib.get(b"native_run").expect("Not load.");
    //     let free_string_func: libloading::Symbol<unsafe extern "C" fn(*const c_char)> =
    //         lib.get(b"native_free_string").expect("Not load.");
    //     //let cancel_func: libloading::Symbol<unsafe extern fn(*mut ExecutionContextHandle)> = lib.get(b"native_cancel").expect("Not load.");

    //     println!("-> Init flow.");

    //     let handle_ptr = Arc::new(Mutex::new(ExecutionContextHandlePtr { ptr: init_func() }));

    //     // TODO: We cannot use cancel_func directly in the handler, since it holds a reference to lib which does not live long enough.
    //     // Thus, we cast directly into ExecutionContext and use the executor's controller directly.
    //     // The downside of this is that the flow must have been compiled with the same Version of ExecutionContext.
    //     let ctx = Box::from_raw(
    //         handle_ptr
    //             .lock()
    //             .unwrap()
    //             .clone()
    //             .ptr
    //             .cast::<ExecutionContext>(),
    //     );
    //     ctrlc::set_handler(move || {
    //         println!("-> Flow execution cancellation requested.");

    //         ctx.executor.controller().lock().unwrap().cancel();

    //         // Does not work...
    //         //cancel_func(mutex_handle_clone.lock().unwrap().ptr);
    //     })
    //     .expect("Error setting Ctrl-C handler.");

    //     println!("-> Start flow execution.");

    //     let result_ptr = run_func(args.workers, handle_ptr.lock().unwrap().ptr);
    //     let result = CStr::from_ptr(result_ptr).to_string_lossy().into_owned();
    //     free_string_func(result_ptr);

    //     println!("-> Flow execution ended.");

    //     println!("-> Flow execution result: {}", result);
    // }
    Ok(())
}
