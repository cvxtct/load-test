use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Debug, Parser, Clone)]
#[command(
    name = "rstress",
    version,
    about = "Tiny HTTP load tester with async + multiprocess"
)]
#[command(group(
    ArgGroup::new("runmode")
        .args(["worker"])
        .multiple(false)
))]
pub struct Cli {
    #[arg(short, long)]
    pub config: PathBuf,

    #[arg(long)]
    pub worker: Option<usize>,
}
