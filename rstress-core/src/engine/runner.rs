use anyhow::{anyhow, Result};
use std::time::Instant;

use crate::{
    cli::Cli,
    config::Config,
    engine::{multiproc::spawn_workers, worker::run_worker},
    metrics::report::print_human,
};

pub async fn run(cli: &Cli, cfg: &Config) -> Result<()> {
    if cfg.processes == 0 {
        return Err(anyhow!("processes must be >= 1"));
    }

    match cli.worker {
        None => {
            if cfg.processes == 1 {
                let t0 = Instant::now();
                let m = run_worker(0, cfg).await?;
                print_human(Some(0), &m, t0.elapsed().as_secs_f64());
            } else {
                spawn_workers(cli, cfg)?;
                let t0 = Instant::now();
                let m = run_worker(0, cfg).await?;
                print_human(Some(0), &m, t0.elapsed().as_secs_f64());
            }
        }
        Some(id) => {
            let t0 = Instant::now();
            let m = run_worker(id, cfg).await?;
            print_human(Some(id), &m, t0.elapsed().as_secs_f64());
        }
    }
    Ok(())
}