use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use rstress_core::{
    cli::Cli,
    config::{load_config, print_config},
    engine::runner::run,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Start a global, human-readable logger that listens to all tracing events,
    // formats them to stdout, and respects the RUST_LOG environment variable (default: info) 
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();
    let cfg = load_config(&cli.config)?;

    println!("Starting load test...");
    print_config(&cfg);

    run(&cli, &cfg).await
}