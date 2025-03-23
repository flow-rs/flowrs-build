use clap::{arg, command, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    /// Shared library file of the flow to run.
    #[arg(short, long)]
    pub flow: String,

    /// Number of workers to use.
    #[arg(short, long, default_value_t = 1)]
    pub workers: u128,

    /// Role of the runtime (orchestrator or node-runtime).
    #[arg(short, long)]
    pub role: String,

    /// Runtime ID (only required for node runtimes)
    #[arg(long)]
    pub runtime_id: Option<u128>,
}
