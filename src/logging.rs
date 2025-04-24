use std::{env, fs::File, io::BufWriter};
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logging() {
    let log_to_file = env::var("LOG_TO_FILE").unwrap_or_default() == "1";

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let make_writer = if log_to_file {
        BoxMakeWriter::new(|| {
            let file = File::create("/var/log/flowrs.log").expect("Failed to create log file");
            BufWriter::new(file)
        })
    } else {
        BoxMakeWriter::new(|| std::io::stdout())
    };

    let fmt_layer = fmt::layer()
        .with_writer(make_writer)
        .with_ansi(!log_to_file);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    tracing::info!("Logging initialized (log_to_file = {})", log_to_file);
}
