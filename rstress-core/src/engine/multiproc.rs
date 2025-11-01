use anyhow::{Context, Result};
use std::process::Command;
use tracing::info;

use crate::cli::Cli;
use crate::config::Config;

/// Spawn N-1 worker processes (master is worker 0).
pub fn spawn_workers(cli: &Cli, cfg: &Config) -> Result<()> {
    for id in 1..cfg.processes {
        let mut cmd = Command::new(std::env::current_exe()?);
        cmd.arg("--config")
            .arg(&cli.config)
            .arg("--worker")
            .arg(id.to_string());

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }

        let child = cmd
            .spawn()
            .with_context(|| format!("spawning worker {id}"))?;
        info!("Spawned worker process {id} (pid={})", child.id());
    }
    Ok(())
}