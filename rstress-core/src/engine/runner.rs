use anyhow::{anyhow, Result};
use std::time::Instant;

use crate::{
    cli::Cli,
    config::Config,
    engine::{multiproc::spawn_workers, worker::run_worker},
    metrics::report::print_human,
};

pub async fn run(cli: &Cli, cfg: &Config) -> Result<()> {
    // Core vs process setting safeguards.
    if cfg.processes == 0 {
        return Err(anyhow!("processes must be >= 1"));
    }

    let max_cores = std::thread::available_parallelism()
        .map(|n|n.get())
        .unwrap_or(1);
    let recommended_max = max_cores / 2;

    if cfg.processes > recommended_max.max(1) {
        return Err(anyhow!(
            "processes={} exceeds recommended maximum ({}) — \
            try <= {} for best performance",
            cfg.processes,
            max_cores,
            recommended_max
    ));
    }

    // if cfg.processes > recommended_max.max(1) {
    // tracing::warn!(
    //     "Configured {} processes exceeds recommended maximum ({} cores total). \
    //      This may reduce performance.",
    //     cfg.processes,
    //     max_cores
    //     );
    // }


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