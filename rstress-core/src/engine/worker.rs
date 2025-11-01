use anyhow::{Context, Result};
use reqwest::Client;
use std::{sync::Arc, time::Instant};
use tokio::{sync::{mpsc, Semaphore}, time};
use tracing::info;

use crate::{
    config::Config,
    http::build_client,
    metrics::metrics::Metrics,
    util::is_ok_status,
    engine::request::{RequestSpec, build_request_spec},
};

/// Run a single worker (one process) and return its Metrics.
/// Matches the original behavior (no streaming IPC yet).
pub async fn run_worker(worker_id: usize, cfg: &Config) -> Result<Metrics> {
    info!("Worker {worker_id} starting…");

    let duration = humantime::parse_duration(&cfg.duration)
        .with_context(|| format!("invalid duration: {}", cfg.duration))?;
    let client: Client = build_client(cfg)?;

    let spec: RequestSpec = build_request_spec(cfg)?;

    // Split global rate across processes
    let base = cfg.rate_per_sec / cfg.processes as u32;
    let rem  = cfg.rate_per_sec % cfg.processes as u32;
    let my_rate = base + if worker_id < rem as usize { 1 } else { 0 };
    if my_rate == 0 {
        info!("Worker {worker_id}: assigned rate is 0 req/s; exiting.");
        return Ok(Metrics::new());
    }

    // Rate limiter & concurrency gate
    let mut limiter = time::interval(time::Duration::from_secs_f64(1.0 / my_rate as f64));
    limiter.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
    let semaphore = Arc::new(Semaphore::new(cfg.concurrency_per_process));

    // Channel for metrics: (ok, latency_us, Option<status_code>)
    let (tx, mut rx) = mpsc::unbounded_channel::<(bool, u64, Option<u16>)>();

    let start_at = Instant::now();
    let end_at = start_at + duration;

    while Instant::now() < end_at {
        limiter.tick().await;

        // backpressure: skip this tick if saturated
        let permit = match semaphore.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let client = client.clone();
        let url = spec.url.clone();
        let headers = spec.headers.clone();
        let method = spec.method.clone();
        let payload = spec.payload.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            let _permit = permit; // keeps semaphore slot until drop
            let req_start = Instant::now();

            let mut req = client.request(method, &url);
            for (k, v) in headers {
                req = req.header(k, v);
            }
            if let Some(body) = payload {
                req = req.body(body);
            }

            let (ok, code) = match req.send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let _ = resp.bytes().await; // drain to reuse connection
                    (is_ok_status(status), Some(status))
                }
                Err(_) => (false, None), // transport error
            };

            let lat_us = req_start.elapsed().as_micros() as u64;
            let _ = tx.send((ok, lat_us, code));
        });
    }

    // Drop our last sender so receiver will finish when tasks are done
    drop(tx);

    // Aggregate metrics
    let mut metrics = Metrics::new();
    while let Some((ok, lat, code)) = rx.recv().await {
        metrics.record(ok, lat, code);
    }

    Ok(metrics)
}