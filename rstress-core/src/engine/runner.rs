use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use std::time::Instant;

use crate::{
    cli::Cli,
    config::Config,
    engine::{multiproc::spawn_workers, worker::run_worker},
    metrics::report::{print_human, write_run_json, WorkerReport},
};

pub async fn run(cli: &Cli, cfg: &Config) -> Result<()> {
    // Core vs process setting safeguards.
    if cfg.processes == 0 {
        return Err(anyhow!("processes must be >= 1"));
    }

    let max_cores = std::thread::available_parallelism()
        .map(|n| n.get())
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

    match cli.worker {
        None => {
            if cfg.processes == 1 {
                // mark start
                let started: DateTime<Utc> = Utc::now();
                let t0 = Instant::now();

                let m = run_worker(0, cfg).await?;
                let elapsed = t0.elapsed().as_secs_f64();

                // print console summary
                print_human(Some(0), &m, elapsed);

                // write JSON report
                let ts = started.format("%Y%m%dT%H%M%SZ").to_string();
                let fname = format!("{}-worker-0.json", ts);
                let rep = WorkerReport::from_metrics(Some(0), &m, started, elapsed);
                let cfg_view = cfg.redacted_for_report();
                write_run_json(fname, &cfg_view, vec![rep])?;
            } else {
                spawn_workers(cli, cfg)?;

                let started: DateTime<Utc> = Utc::now();
                let t0 = Instant::now();

                let m = run_worker(0, cfg).await?;
                let elapsed = t0.elapsed().as_secs_f64();

                print_human(Some(0), &m, elapsed);

                let ts = started.format("%Y%m%dT%H%M%SZ").to_string();
                let fname = format!("{}-worker-0.json", ts);
                let rep = WorkerReport::from_metrics(Some(0), &m, started, elapsed);
                let cfg_view = cfg.redacted_for_report();
                write_run_json(fname, &cfg_view, vec![rep])?;
            }
        }

        Some(id) => {
            let started: DateTime<Utc> = Utc::now();
            let t0 = Instant::now();

            let m = run_worker(id, cfg).await?;
            let elapsed = t0.elapsed().as_secs_f64();

            print_human(Some(id), &m, elapsed);

            let ts = started.format("%Y%m%dT%H%M%SZ").to_string();
            let fname = format!("{}-worker-{}.json", ts, id);
            let rep = WorkerReport::from_metrics(Some(id), &m, started, elapsed);
            let cfg_view = cfg.redacted_for_report();
            write_run_json(fname, &cfg_view, vec![rep])?;
        }
    }

    Ok(())
}
