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

pub fn print_startup_banner() {
    let banner = r#"
________/\\\\\__/\\\\\\_____________________________________________________________________________        
 ______/\\\///__\////\\\_____________________________________________________________________________       
  _____/\\\_________\/\\\_____________________________________________________________________________      
   __/\\\\\\\\\______\/\\\________/\\\\\_____/\\____/\\___/\\__/\\\\\\\\\\\__/\\/\\\\\\\___/\\\\\\\\\\_     
    _\////\\\//_______\/\\\______/\\\///\\\__\/\\\__/\\\\_/\\\_\///////////__\/\\\/////\\\_\/\\\//////__    
     ____\/\\\_________\/\\\_____/\\\__\//\\\_\//\\\/\\\\\/\\\________________\/\\\___\///__\/\\\\\\\\\\_   
      ____\/\\\_________\/\\\____\//\\\__/\\\___\//\\\\\/\\\\\_________________\/\\\_________\////////\\\_  
       ____\/\\\_______/\\\\\\\\\__\///\\\\\/_____\//\\\\//\\\__________________\/\\\__________/\\\\\\\\\\_ 
        ____\///_______\/////////_____\/////________\///__\///___________________\///__________\//////////__
"#;

    // Color codes
    let blue_1 = "\x1b[38;5;27m";
    let blue_2 = "\x1b[38;5;33m";
    let blue_3 = "\x1b[38;5;39m";
    let blue_4 = "\x1b[38;5;45m";
    let blue_5 = "\x1b[38;5;51m";
    let reset = "\x1b[0m";

    // Split and color lines progressively
    let lines: Vec<&str> = banner.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let color = match i {
            0..=1 => blue_1,
            2..=3 => blue_2,
            4..=5 => blue_3,
            6..=6 => blue_4,
            _ => blue_5,
        };
        println!("{}{}{}", color, line, reset);
    }

    println!(); // Spacer
    tracing::info!("🌸 flow-rs runtime is starting up");
    tracing::info!("🔗 https://github.com/flow-rs");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_banner() {
        print_startup_banner();
        // This is a visual/logging test to ensure the banner prints without errors.
        // You can check the output manually, or redirect stdout for verification in advanced setups.
    }
}
