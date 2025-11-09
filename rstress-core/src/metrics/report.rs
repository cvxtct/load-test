// report.rs
use super::{aggregate::quantile_ms, metrics::Metrics};
use serde::Serialize;
use std::{fs, path::Path};
use chrono::{DateTime, Utc};


pub fn print_json(m: &Metrics) -> String {
    #[derive(serde::Serialize)]
    struct MetricsJson<'a> {
        sent: u64,
        ok: u64,
        err: u64,
        dropped: u64,
        codes: &'a std::collections::BTreeMap<u16, u64>,
        transport: &'a std::collections::BTreeMap<&'static str, u64>,
        p50_ms: f64,
        p95_ms: f64,
        p99_ms: f64,
    }

    serde_json::to_string_pretty(&MetricsJson {
        sent: m.sent,
        ok: m.ok,
        err: m.err,
        dropped: m.dropped,
        codes: &m.codes,
        transport: &m.transport,
        p50_ms: crate::metrics::aggregate::quantile_ms(&m.hist, 50.0),
        p95_ms: crate::metrics::aggregate::quantile_ms(&m.hist, 95.0),
        p99_ms: crate::metrics::aggregate::quantile_ms(&m.hist, 99.0),
    }).unwrap()
}

pub fn print_human(worker: Option<usize>, m: &Metrics, elapsed_s: f64) {
    let rps = m.sent as f64 / elapsed_s.max(1e-6);

    match worker {
        Some(id) => println!(
            "[worker {id}] sent={:<6} ok={:<6} err={:<6} rps={:<8.1} p50={:<7.2}ms p95={:<7.2}ms p99={:<7.2}ms",
            m.sent, m.ok, m.err, rps,
            quantile_ms(&m.hist, 50.0),
            quantile_ms(&m.hist, 95.0),
            quantile_ms(&m.hist, 99.0),
        ),
        None => println!(
            "sent={:<6} ok={:<6} err={:<6} rps={:<8.1} p50={:<7.2}ms p95={:<7.2}ms p99={:<7.2}ms",
            m.sent, m.ok, m.err, rps,
            quantile_ms(&m.hist, 50.0),
            quantile_ms(&m.hist, 95.0),
            quantile_ms(&m.hist, 99.0),
        ),
    }

    if !m.codes.is_empty() {
        println!("status codes:");
        for (code, count) in &m.codes {
            if *code == 0 { println!("  0 (Transport Errors): {}", count); }
            else { println!("  {}: {}", code, count); }
        }
    }
    if !m.transport.is_empty() {
        println!("Transport error.");
        for (k, v) in &m.transport {
            println!("  {k}: {v}");
        }
    }
    if m.dropped > 0 {
        print!("Dropped due to low resource: {}", m.dropped);
    }
}

#[derive(Serialize)]
pub struct WorkerReport<'a> {
    pub worker_id: Option<usize>,
    pub started_at: String,
    pub elapsed_s: f64,
    pub sent: u64,
    pub ok: u64,
    pub err: u64,
    pub dropped: u64,
    pub rps: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub codes: &'a std::collections::BTreeMap<u16, u64>,
    pub transport: &'a std::collections::BTreeMap<&'static str, u64>,
}

impl<'a> WorkerReport<'a> {
    pub fn from_metrics(worker_id: Option<usize>, m: &'a Metrics, started_at: DateTime<Utc>, elapsed_s: f64) -> Self {
        let rps = m.sent as f64 / elapsed_s.max(1e-6);
        Self {
            worker_id,
            started_at: started_at.to_rfc3339(),
            elapsed_s,
            sent: m.sent,
            ok: m.ok,
            err: m.err,
            dropped: m.dropped,
            rps,
            p50_ms: quantile_ms(&m.hist, 50.0),
            p95_ms: quantile_ms(&m.hist, 95.0),
            p99_ms: quantile_ms(&m.hist, 99.0),
            codes: &m.codes,
            transport: &m.transport,
        }
    }
}

#[derive(Serialize)]
pub struct RunReport<'a, C: Serialize> {
    pub run_id: String,              // ISO timestamp string
    pub config: &'a C,               // snapshot (redacted or full)
    pub workers: Vec<WorkerReport<'a>>,
}

pub fn write_run_json<C: Serialize, P: AsRef<Path>>(
    path: P,
    config: &C,
    workers: Vec<WorkerReport<'_>>,
) -> std::io::Result<()> {
    let run_id = chrono::Utc::now().to_rfc3339();
    let doc = RunReport { run_id, config, workers };
    let s = serde_json::to_string_pretty(&doc).expect("serialize run report");
    fs::write(path, s)
}

