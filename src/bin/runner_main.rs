use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use tokio::net::lookup_host;

use anyhow::Error;
use clap::Parser;
use flowrs::comm::communication::Communicator;
use flowrs::comm::network_communicator::NetworkCommunicator;
use flowrs::exec::execution::{Executor, StandardExecutor};
use flowrs::flow::abstract_flow::AbstractFlow;

use flowrs::comm::messages::Message;
use flowrs::exec::execution_configuration::ExecutionConfig;
use flowrs::exec::execution_configuration::NodeConfig;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::task;
use tokio::time::{sleep, Duration};

// port constants
const SETUP_PORT: u16 = 4999; // used to establish connections between node-runtimes and the orchestrator
const RUNTIME_PORT: u16 = 5000; // used to communicate TO any node-runtime

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

    // create dummy values for now
    let abstract_flow = return_dummy_flow()?;
    //let orchestrator_addr =
    //SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
    //let orchestrator_addr = "orchestrator:5000".parse().unwrap(); //find IP using docker
    let orchestrator_addr = get_orchestrator_address().await?; // find the IP using tokio

    println!(
        "Orchestrator Address: {}",
        orchestrator_addr.ip().to_string()
    );

    // Branch logic based on the role argument
    match args.role.as_str() {
        "orchestrator" => {
            run_orchestrator(args, abstract_flow).await?;
        }
        "node-runtime" => {
            run_node_runtime(args, abstract_flow, orchestrator_addr).await?;
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

// Logic for running as orchestrator
async fn run_orchestrator(args: Arguments, abstract_flow: AbstractFlow) -> Result<(), Error> {
    // TODO: add number of runtimes to config and pass as parameter
    let number_of_runtimes_expected = 2;

    println!(
        "[Orchestrator] Running as orchestrator with flow file: {}",
        args.flow
    );

    // =======================================================================================
    // Step 1: Connect to node runtimes
    // =======================================================================================

    // start listening for incoming runtime connections
    let listener = TcpListener::bind(format!("0.0.0.0:{}", SETUP_PORT).as_str()).await?;
    println!(
        "[Orchestrator] Listening for node runtimes on port {}...",
        SETUP_PORT
    );

    let connected_runtimes = std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::<
        String,
        (NetworkCommunicator, NetworkCommunicator, u16),
    >::new()));
    let mut next_port = RUNTIME_PORT + 1; // start port assignments from the first free port

    while connected_runtimes.lock().await.len() < number_of_runtimes_expected {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let runtime_ip = addr.ip().to_string();
                println!(
                    "[Orchestrator] Node runtime connected from {}...",
                    runtime_ip
                );

                let mut assigned_port = next_port;
                let mut port_should_increment = true;

                let mut lock = connected_runtimes.lock().await;
                if let Some((old_sender, old_receiver, old_port)) = lock.remove(&runtime_ip) {
                    println!(
                        "[Orchestrator] Detected reconnection from {}. Cleaning up old connections...",
                        runtime_ip
                    );
                    drop(old_sender);
                    drop(old_receiver);

                    assigned_port = old_port;
                    port_should_increment = false;
                }

                // **Oneshot channel to signal receiver readiness**
                let (tx, rx) = oneshot::channel::<Result<NetworkCommunicator, Error>>();

                // **Spawn Receiver First**
                let connected_runtimes_clone = std::sync::Arc::clone(&connected_runtimes);
                let receiver_runtime_ip = runtime_ip.clone();
                let receiver_task = task::spawn(async move {
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
                            let _ = tx.send(Ok(receiver)); // Notify sender that receiver is ready
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
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                }

                println!(
                    "[Orchestrator] Assigning port {} to runtime {}...",
                    assigned_port, runtime_ip
                );

                let setup_msg = Message::<String>::SetupCommunicationPort(assigned_port);
                println!("[Orchestrator] Sending message: {:?}", setup_msg); // Log the message
                if let Err(e) = sender.send(setup_msg).await {
                    println!(
                        "[Orchestrator] Failed to send port assignment message to {}: {}",
                        runtime_ip, e
                    );
                    continue;
                }

                println!(
                    "[Orchestrator] Sent SetupCommunicationPort message to {} for port {}",
                    runtime_ip, assigned_port
                );
                sleep(Duration::from_millis(100)).await; // Give time for the message to be processed

                // **Wait for Receiver to be Ready**
                let receiver = match rx.await {
                    Ok(Ok(r)) => r,
                    Ok(Err(e)) => {
                        println!("[Orchestrator] Receiver failed: {}", e);
                        continue;
                    }
                    Err(_) => {
                        println!("[Orchestrator] Receiver channel closed unexpectedly");
                        continue;
                    }
                };

                // **Store the communicator in the HashMap**
                let mut lock = connected_runtimes_clone.lock().await;
                lock.insert(runtime_ip.clone(), (sender, receiver, assigned_port));

                if port_should_increment {
                    next_port += 1
                };

                println!(
                    "[Orchestrator] Successfully registered runtime {} with port {}. [{} Total from {} Expected]",
                    runtime_ip, assigned_port,
                    lock.len(),
                    number_of_runtimes_expected,
                );
            }
            Err(e) => {
                println!("[Orchestrator] Failed to accept connection: {}", e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    // // Step 1: Load flow data
    // // Redesign of flowrs-build code generation necessary
    // // for now asume that flow data is given with the new flow data structure
    // //let abstract_flow = return_dummy_flow()?;
    // // Step 2: Load Environment Configuration
    // // Step 3: Run Scheduler -> provides execution_config

    // //      create DUMMY execution config
    // let mut execution_config = ExecutionConfig::new();
    // // set all node configs to local because it is not yet implemented
    // execution_config.node_configs = abstract_flow
    //     .get_nodes()
    //     .map(|(node_id, _node)| (*node_id, NodeConfig::LocalNodeConfig))
    //     .collect();

    // // Step 4: Run Executor
    // let mut executor = StandardExecutor::new();
    // executor
    //     .setup_and_connect(abstract_flow, execution_config)
    //     .await?;
    // // Step 5: Clean up
    println!("[Orchestrator] All expected runtimes connected!");
    Ok(())
}

/// Logic for running as a node runtime
async fn run_node_runtime(
    args: Arguments,
    abstract_flow: AbstractFlow,
    orch_addr: SocketAddr,
) -> Result<(), Error> {
    use flowrs::comm::messages::Message;

    println!(
        "[Node RT] Running as node runtime with flow file: {}",
        args.flow
    );

    let orchestrator_ip = orch_addr.ip().to_string();
    let assigned_port: u16;

    // Create Oneshot channel to return receiver from task spawned in step 1
    let (tx, rx) = oneshot::channel();

    // =======================================================================================
    // Step 1: Create a receiver communicator to receive on RUNTIME_PORT
    // =======================================================================================
    let recv_task = task::spawn(async move {
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
                // Send receiver back so we can use it later
                let _ = tx.send(orch_receiver);
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
    let max_retries = 5;

    while retries < max_retries {
        match TcpStream::connect((orchestrator_ip.clone(), SETUP_PORT)).await {
            Ok(_stream) => {
                println!(
                    "[Node RT] Successfully signaled presence to orchestrator at {}:{}",
                    orchestrator_ip, SETUP_PORT
                );
                break;
            }
            Err(_) => {
                println!(
                    "[Node RT] Failed to connect to orchestrator setup port. Retrying... ({}/{})",
                    retries + 1,
                    max_retries
                );
                retries += 1;
                sleep(Duration::from_secs(1)).await;
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
    // Wait for the receiver task to complete and retrieve the communicator
    println!("[Node RT] Waiting for assigned communication port from orchestrator...");
    let mut orch_receiver = rx.await.expect("should receive");
    let assigned_port_msg: Message<String> = orch_receiver.receive().await.expect("should receive");

    if let Message::SetupCommunicationPort(port) = assigned_port_msg {
        assigned_port = port;
        println!(
            "[Node RT] Received assigned port from orchestrator: {}",
            assigned_port
        );
    } else {
        return Err(anyhow::Error::msg(
            "[Node RT] Unexpected message received instead of assigned port!",
        ));
    }

    println!("[Node RT] Received message: {:?}", assigned_port_msg);

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

    println!(
        "[Node RT] Successfully connected to orchestrator on {}:{} (send) and {}:{} (receive)",
        orchestrator_ip, assigned_port, orchestrator_ip, RUNTIME_PORT
    );

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
