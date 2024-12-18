use clap::Parser;
use flowrs::exec::execution::{ExecutionContext, ExecutionContextHandle, Executor};
use std::ffi::{c_char, CStr};
use std::sync::{Arc, Mutex};

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

#[derive(Clone)]
struct ExecutionContextHandlePtr {
    ptr: *mut ExecutionContextHandle,
}
unsafe impl Send for ExecutionContextHandlePtr {}

fn main() {
    // Define the CLI application using clap
    let args = Arguments::parse();
    // Branch logic based on the role argument
    match args.role.as_str() {
        "orchestrator" => {
            run_orchestrator(args);
        }
        "node-runtime" => {
            run_node(args);
        }
        _ => {
            eprintln!("Invalid role specified. Use 'orchestrator' or 'node-runtime'.");
            std::process::exit(1);
        }
    }
}

fn return_dummy_flow() {
    //Define communicators
    let mut node1_comm =
        NodeCommunicator::ThreadComm(ThreadCommunicator::<u32>::new().expect("should construct"));
    let mut node2_comm =
        NodeCommunicator::ThreadComm(ThreadCommunicator::<u32>::new().expect("should construct"));
    let output_comm =
        NodeCommunicator::ThreadComm(ThreadCommunicator::<u32>::new().expect("should construct"));

    //Define nodes
    let number_node_1: ValueNode<u32> = ValueNode::<u32>::new(3, node1_comm.clone_send());
    let number_node_2: ValueNode<u32> = ValueNode::<u32>::new(2, node2_comm.clone_send());
    let add_node = AddNode::<u32, u32, u32>::new(
        node1_comm.move_recv().expect("should move"),
        node2_comm.move_recv().expect("should move"),
        output_comm.clone_send(),
    );
    //Define flow
    let mut flow = Flow::new_empty();
    flow.add_node_with_id(number_node_1, 1);
    flow.add_node_with_id(number_node_2, 2);
    flow.add_node_with_id(add_node, 3);
    flow.connect_nodes(1, 3, 0, 0);
    flow.connect_nodes(2, 3, 0, 1);
}

// Logic for running as orchestrator
fn run_orchestrator(args: Arguments) {
    println!("Running as orchestrator with flow file: {}", args.flow);

    // Step 1: Load flow data
    // Redesign of flowrs-build code generation necessary
    // for now asume that flow data is given with the new flow data structure
    // Step 2: Load Environment Configuration
    // Step 3: Run Scheduler
    // Step 4: Run Executor
    // Step 5: Clean up
}

/// Logic for running as a node
fn run_node(args: Arguments) {
    println!("Running as node runtime with flow file: {}", args.flow);

    // Load the dynamic library
    println!("-> Load flow from {}.", args.flow);

    // Step 1: Load flow data
    // Step 2: Start local nodes
    // Step 3: Await connection from orchestrator
    // Step 4: Run Nodes
    // Step 5: Clean up
    unsafe {
        let lib = libloading::Library::new(args.flow).expect("Failed to load the dynamic library");

        let init_func: libloading::Symbol<unsafe extern "C" fn() -> *mut ExecutionContextHandle> =
            lib.get(b"native_init").expect("Not load.");
        let run_func: libloading::Symbol<
            unsafe extern "C" fn(usize, *mut ExecutionContextHandle) -> *const c_char,
        > = lib.get(b"native_run").expect("Not load.");
        let free_string_func: libloading::Symbol<unsafe extern "C" fn(*const c_char)> =
            lib.get(b"native_free_string").expect("Not load.");
        //let cancel_func: libloading::Symbol<unsafe extern fn(*mut ExecutionContextHandle)> = lib.get(b"native_cancel").expect("Not load.");

        println!("-> Init flow.");

        let handle_ptr = Arc::new(Mutex::new(ExecutionContextHandlePtr { ptr: init_func() }));

        // TODO: We cannot use cancel_func directly in the handler, since it holds a reference to lib which does not live long enough.
        // Thus, we cast directly into ExecutionContext and use the executor's controller directly.
        // The downside of this is that the flow must have been compiled with the same Version of ExecutionContext.
        let ctx = Box::from_raw(
            handle_ptr
                .lock()
                .unwrap()
                .clone()
                .ptr
                .cast::<ExecutionContext>(),
        );
        ctrlc::set_handler(move || {
            println!("-> Flow execution cancellation requested.");

            ctx.executor.controller().lock().unwrap().cancel();

            // Does not work...
            //cancel_func(mutex_handle_clone.lock().unwrap().ptr);
        })
        .expect("Error setting Ctrl-C handler.");

        println!("-> Start flow execution.");

        let result_ptr = run_func(args.workers, handle_ptr.lock().unwrap().ptr);
        let result = CStr::from_ptr(result_ptr).to_string_lossy().into_owned();
        free_string_func(result_ptr);

        println!("-> Flow execution ended.");

        println!("-> Flow execution result: {}", result);
    }
}
