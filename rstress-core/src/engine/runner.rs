use anyhow::{anyhow, Result};
use std::time::Instant;

use crate::{
    cli::Cli,
    config::Config,
    engine::{multiproc::spawn_workers, worker::run_worker},
    metrics::report::print_human,
};

/// Top-level run: either spawn sub-processes and run worker 0,
/// or run single worker when processes == 1.
/// (Per your original behavior; each worker prints its own summary.)
pub async fn run(cli: &Cli, cfg: &Config) -> Result<()> {
    if cfg.processes == 0 {
        return Err(anyhow!("processes must be >= 1"));
    }

    match cli.worker {
        None => {
            if cfg.processes == 1 {
                let t0 = Instant::now();
                let m = run_worker(0, cfg).await?;
                let elapsed_s = t0.elapsed().as_secs_f64();
                print_human(Some(0), &m, elapsed_s);
            } else {
                spawn_workers(cli, cfg)?;
                // Master also participates as worker 0
                let t0 = Instant::now();
                let m = run_worker(0, cfg).await?;
                let elapsed_s = t0.elapsed().as_secs_f64();
                print_human(Some(0), &m, elapsed_s);
            }
        }
        Some(_id) => {
            let t0 = Instant::now();
            let m = run_worker(0, cfg).await?;
            let elapsed_s = t0.elapsed().as_secs_f64();
            print_human(Some(0), &m, elapsed_s);
        }
    }
    Ok(())
}