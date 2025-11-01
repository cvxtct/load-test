use clap::{ArgGroup, Parser};
use std::path::PathBuf;

/// CLI options for rstress (shared so tests/tools can reuse).
#[derive(Debug, Parser, Clone)]
#[command(name = "rstress", version, about = "Tiny HTTP load tester with async + multiprocess")]
#[command(group(
    ArgGroup::new("runmode")
        .args(["worker"])
        .multiple(false)
))]
pub struct Cli {
    /// Path to config file (YAML or TOML)
    #[arg(short, long)]
    pub config: PathBuf,

    /// Internal: run as a worker process with this ID (0..processes-1)
    #[arg(long)]
    pub worker: Option<usize>,
}