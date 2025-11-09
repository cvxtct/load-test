use anyhow::{Context, Result};
use bytes::Bytes;
use reqwest::Client;
use std::{sync::Arc, time::Instant, u64};
use tokio::{sync::{mpsc, Semaphore}, time};
use tokio::time::{timeout, Duration};
use tracing::info;

use crate::{
    config::Config,
    http::build_client,
    metrics::metrics::Metrics,
    util::{is_ok_status, classify_reqwest_error},
    engine::request::{RequestSpec, build_request_spec},
};

/// Run a single worker (one process) and return its Metrics.
pub async fn run_worker(worker_id: usize, cfg: &Config) -> Result<Metrics> {
    info!("Worker {worker_id} starting…");

    let duration = humantime::parse_duration(&cfg.duration)
        .with_context(|| format!("invalid duration: {}", cfg.duration))?;
    let client: Client = build_client(cfg)?;
    let spec: RequestSpec = build_request_spec(cfg)?;

    // Split global rate across processes
    let base = cfg.rate_per_sec / cfg.processes as u32;
    let rem  = cfg.rate_per_sec % cfg.processes as u32;
    let calced_rate = base + if worker_id < rem as usize { 1 } else { 0 };

    if calced_rate == 0 {
        info!("Worker {worker_id}: assigned rate is 0 req/s; exiting.");
        return Ok(Metrics::new());
    }

    // Prepare cheap-to-clone parts
    let url = Arc::new(spec.url);
    let method = spec.method;
    let headers = Arc::new(spec.headers);
    let payload: Option<Arc<Bytes>> = spec.payload.map(|p| Arc::new(Bytes::from(p)));

    // Rate limiter & concurrency gate
    // NOTE: for very high rates, limiter rates could be int instead of float.
    let mut limiter = time::interval(time::Duration::from_secs_f64(1.0 / calced_rate as f64));
    limiter.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
    let semaphore = Arc::new(Semaphore::new(cfg.concurrency_per_process));
    // NOTE: chanel buffer can be 16384
    let (tx, mut rx) = mpsc::channel::<(bool, u64, Option<u16>, Option<&'static str>, u64)>(4096);

    let start_at = Instant::now();
    let end_at = start_at + duration;

    let mut dropped: u64 = 0;

    while Instant::now() < end_at {
        limiter.tick().await;

        let permit = match semaphore.clone().try_acquire_owned() { // or acquire_owned()
            Ok(p) => p,
            Err(_) => {
                    dropped += 1;
                    continue;
                    } // This skip request if permit is not available
        };

        let client = client.clone();
        let url = url.clone();
        let headers = headers.clone();
        let payload = payload.clone();
        let tx = tx.clone();

        // clone here so each task owns its own Method
        let method = method.clone();

        let timeout_r = cfg.timeout_r;

        tokio::spawn(async move {
            let _permit = permit;
            let t0 = Instant::now();

            let mut req = client.request(method, &*url);
            for (k, v) in headers.iter().cloned() {
                req = req.header(k, v);
            }
            if let Some(body) = &payload {
                req = req.body(body.as_ref().clone());
            }

            // Simulate request based timeout
            let res = timeout(Duration::from_secs(timeout_r), req.send()).await;

            let (ok, code, kind_opt) = match res {
                Ok(Ok(resp)) => {
                    let status = resp.status().as_u16();
                    let _ = resp.bytes().await; // drain to reuse connection
                    (is_ok_status(status), Some(status), None)
                }
                // The request itself failed (connection, DNS, etc.)
                Ok(Err(e)) => (false, None, Some(classify_reqwest_error(&e))),
                // The timeout fired
                Err(_elapsed) => (false, None, Some("request_timeout")),
            };

            let lat_us = t0.elapsed().as_micros() as u64;
            let _ = tx.send((ok, lat_us, code, kind_opt, dropped)).await;
        });
    }

    drop(tx); // close channel so receiver terminates

    let mut metrics = Metrics::new();
    while let Some((ok, lat, code, kind_opt, dropped)) = rx.recv().await {
        metrics.record(ok, lat, code);
        if let Some(kind) = kind_opt {
            metrics.record_transport_kind(kind);
        }
        if dropped > 0 {
            metrics.record_dropped(dropped);
        }
    }

    Ok(metrics)
}