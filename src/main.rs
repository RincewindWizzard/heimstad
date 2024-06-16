#![doc = include_str!("../README.md")]


mod mqtt;
mod just_channels;
mod actor;


use clap::Parser;
use log::{debug, Level};

/// A web UI for my smart home
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Log level
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    stderrlog::new()
        .module(module_path!())
        .quiet(args.verbose.is_silent())
        .verbosity(args.verbose.log_level().unwrap_or(Level::Info)) // show warnings and above
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
        .expect("Could not setup logging!");

    debug!("Args: {:?}", args);
    let version = env!("CARGO_PKG_VERSION");
    println!("You are running heimstad {version}");

    //mqtt::mqtt_connect().await;
    let _ = crate::just_channels::main().await;
}
