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
}

#[derive(Clone)]
struct ExecutionContextHandlePtr {
    ptr: *mut ExecutionContextHandle,
}
unsafe impl Send for ExecutionContextHandlePtr {}

fn main() {
    // Define the CLI application using clap
    let args = Arguments::parse();

    // Load the dynamic library
    println!("-> Load flow from {}.", args.flow);

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
