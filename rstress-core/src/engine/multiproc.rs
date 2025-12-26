use anyhow::{Context, Result};
use std::process::Command;
use tracing::info;

use crate::{cli::Cli, config::Config};

pub fn spawn_workers(cli: &Cli, cfg: &Config) -> Result<()> {
    for id in 1..cfg.processes {
        // no ID 0 spawn!
        let mut cmd = Command::new(std::env::current_exe()?);
        cmd.arg("--config")
            .arg(&cli.config)
            .arg("--worker")
            .arg(id.to_string());

        // Forward only the env var RUST_LOG if present
        if let Ok(val) = std::env::var("RUST_LOG") {
            cmd.env("RUST_LOG", val);
        }
        // Or forward everything (comment one or the other)
        // cmd.envs(std::env::vars());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        // These lines control the makeing of child process, inherit parent process context,
        // env vars, etc.
        let child = cmd
            .spawn()
            .with_context(|| format!("spawning worker {id}"))?;
        info!("Spawned worker process {id} (pid={})", child.id());
    }
    Ok(())
}
