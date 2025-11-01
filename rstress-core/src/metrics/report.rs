use super::{aggregate::quantile_ms, metrics::Metrics};
use serde::Serialize;

pub fn print_human(worker: Option<usize>, m: &Metrics, elapsed_s: f64) {
    let rps = m.sent as f64 / elapsed_s.max(1e-6);
    match worker {
        Some(id) => println!(
            "[worker {id}] sent={} ok={} err={} rps={:.1} p50={:.2}ms p95={:.2}ms p99={:.2}ms",
            m.sent, m.ok, m.err, rps,
            quantile_ms(&m.hist, 50.0),
            quantile_ms(&m.hist, 95.0),
            quantile_ms(&m.hist, 99.0),
        ),
        None => println!(
            "sent={} ok={} err={} rps={:.1} p50={:.2}ms p95={:.2}ms p99={:.2}ms",
            m.sent, m.ok, m.err, rps,
            quantile_ms(&m.hist, 50.0),
            quantile_ms(&m.hist, 95.0),
            quantile_ms(&m.hist, 99.0),
        ),
    }
    if !m.codes.is_empty() {
        println!("status codes:");
        for (code, count) in &m.codes {
            if *code == 0 { println!("  0 (transport errors): {}", count); }
            else { println!("  {}: {}", code, count); }
        }
    }
}

#[derive(Serialize)]
struct MetricsJson<'a> {
    sent: u64,
    ok: u64,
    err: u64,
    codes: &'a std::collections::BTreeMap<u16, u64>,
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
}

pub fn print_json(m: &Metrics) -> String {
    serde_json::to_string_pretty(&MetricsJson {
        sent: m.sent,
        ok: m.ok,
        err: m.err,
        codes: &m.codes,
        p50_ms: quantile_ms(&m.hist, 50.0),
        p95_ms: quantile_ms(&m.hist, 95.0),
        p99_ms: quantile_ms(&m.hist, 99.0),
    }).unwrap()
}