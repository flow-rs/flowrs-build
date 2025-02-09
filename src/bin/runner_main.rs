use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use anyhow::Error;
use clap::Parser;
use flowrs::comm::communication::Communicator;
use flowrs::comm::network_communicator::NetworkCommunicator;
use flowrs::exec::execution::{Executor, StandardExecutor};
use flowrs::flow::abstract_flow::AbstractFlow;

use flowrs::exec::execution_configuration::ExecutionConfig;
use flowrs::exec::execution_configuration::NodeConfig;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Define the CLI application using clap
    let args = Arguments::parse();

    // create dummy values for now
    let abstract_flow = return_dummy_flow()?;
    let orchestrator_addr =
        SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);

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
    println!(
        "[Orchestrator] Running as orchestrator with flow file: {}",
        args.flow
    );

    // =======================================================================================
    // Step 1: Connect to node runtimes
    // =======================================================================================
    let listener = TcpListener::bind("0.0.0.0:5000").await?;
    println!("[Orchestrator] Listening for node runtimes on port 5000...");

    let mut connected_runtimes: HashMap<String, NetworkCommunicator> = HashMap::new();
    let mut retries = 0;
    let max_retries = 5;

    while retries < max_retries {
        // wait for node runtimes to connect
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("[Orchestrator] Node runtime connected from {}", addr);

                // create new communicator
                let mut communicator = NetworkCommunicator::new().await.expect("should construct");
                // connect communicator to node runtime for sending
                let connect_res =
                    <NetworkCommunicator as Communicator<String>>::connect_send::<'_, '_>(
                        &mut communicator,
                        Some(addr.ip().to_string()),
                        Some(addr.port()),
                    )
                    .await
                    .expect(
                        format!(
                            "[Orchestrator] Failed to connect to runtime at {}:{}",
                            addr.ip().to_string(),
                            addr.port()
                        )
                        .as_str(),
                    );

                connected_runtimes.insert(addr.to_string(), communicator);
                retries = 0;
            }
            Err(e) => {
                println!(
                    "[Orchestrator] No new connections. Retrying... ({}/{})",
                    retries + 1,
                    max_retries
                );
                retries += 1;
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    if connected_runtimes.is_empty() {
        println!("[Orchestrator] No node runtimes connected. Shutting down.");
        return Err(anyhow::Error::msg("No Nodes Connected"));
    }

    println!("[Orchestrator] Runtime connection successful.");

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

    Ok(())
}

/// Logic for running as a node runtime
async fn run_node_runtime(
    args: Arguments,
    abstract_flow: AbstractFlow,
    orch_addr: SocketAddr,
) -> Result<(), Error> {
    println!(
        "[Node RT] Running as node runtime with flow file: {}",
        args.flow
    );

    // =======================================================================================
    // Step 1: Connect to orchestrator
    // =======================================================================================
    let mut orch_sender = NetworkCommunicator::new().await.expect("should construct");
    let mut orch_receiver = NetworkCommunicator::new().await.expect("should construct");
    let mut retries_sender = 0;
    let mut retries_receiver = 0;
    let max_retries = 5;

    while retries_sender < max_retries {
        match <NetworkCommunicator as Communicator<String>>::connect_send::<'_, '_>(
            &mut orch_sender,
            Some(orch_addr.ip().to_string()),
            Some(orch_addr.port()),
        )
        .await
        {
            Ok(_) => {
                sleep(Duration::from_millis(100)).await; //short wait to ensure orchestrator is ready
                while retries_receiver < max_retries {
                    match <NetworkCommunicator as Communicator<String>>::connect_recv::<'_, '_>(
                        &mut orch_receiver,
                        Some(orch_addr.ip().to_string()),
                        Some(orch_addr.port()),
                    )
                    .await
                    {
                        Ok(_) => {
                            println!(
                                "[Node RT] Connected to orchestrator at {}:{}",
                                orch_addr.ip().to_string(),
                                orch_addr.port()
                            );
                            break;
                        }
                        Err(_) => {
                            println!(
                    "[Node RT] Failed to connect receiver to orchestrator. Retrying... ({}/{})",
                    retries_receiver + 1,
                    max_retries
                );
                            retries_receiver += 1;
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
            Err(_) => {
                println!(
                    "[Node RT] Failed to connect sender to orchestrator. Retrying... ({}/{})",
                    retries_sender + 1,
                    max_retries
                );
                retries_sender += 1;
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

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
