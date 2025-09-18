use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
    process::Command,
    sync::Arc,
    time::Instant,
};

use anyhow::{anyhow, Context, Result};
use clap::{ArgGroup, Parser};
use hdrhistogram::Histogram;
use reqwest::{Client, Method};
use serde::Deserialize;
use tokio::{
    sync::{mpsc, Semaphore},
    time,
};
use tracing::info;
use tracing_subscriber::EnvFilter;

/// Command-line interface options for the load tester.
/// 
/// This struct defines the arguments that can be passed to the program, including:
/// - `config`: Path to the configuration file.
/// - `worker`: Optional worker ID for internal use.
#[derive(Debug, Parser)]
#[command(name = "rstress", version, about = "Tiny HTTP load tester with async + multiprocess")]
#[command(group(
    ArgGroup::new("runmode")
        .args(["worker"])
        .multiple(false)
))]
struct Cli {
    /// Path to config file (YAML or TOML)
    #[arg(short, long)]
    config: PathBuf,

    /// Internal: run as a worker process with this ID (0..processes-1)
    #[arg(long)]
    worker: Option<usize>,
}

/// Configuration options for the load tester.
/// 
/// This struct is deserialized from the configuration file and contains:
/// - `target_url`: The URL to target.
/// - `method`: HTTP method (e.g., GET, POST).
/// - `headers`: Optional HTTP headers.
/// - `payload_file`: Optional path to a file containing the request payload.
/// - `duration`: Duration of the test (e.g., "30s").
/// - `rate_per_sec`: Total request rate per second.
/// - `concurrency_per_process`: Number of concurrent requests per process.
/// - `processes`: Number of OS processes to spawn.
/// - `timeout`: Timeout for each request.
/// - `verify_tls`: Whether to verify TLS certificates.
#[derive(Debug, Deserialize, Clone)]
struct Config {
    target_url: String,
    method: String, // "GET" or "POST"
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    payload_file: Option<String>,
    duration: String,               // e.g. "30s"
    rate_per_sec: u32,              // total rate (master splits across workers)
    concurrency_per_process: usize, // async concurrency per process
    processes: usize,               // number of OS processes
    timeout: String,                // per-request timeout
    #[serde(default = "default_verify_tls")]
    verify_tls: bool,
}

/// Default value for the `verify_tls` field in the `Config` struct.
/// 
/// This function returns `true` to enable TLS verification by default.
fn default_verify_tls() -> bool {
    true
}

/// Metrics for tracking the performance of the load test.
/// 
/// This struct includes:
/// - `sent`: Total number of requests sent.
/// - `ok`: Total number of successful requests.
/// - `err`: Total number of failed requests.
/// - `hist`: Histogram for tracking request latencies.
/// - `codes`: Map of HTTP status codes to their counts.
#[derive(Clone)]
struct Metrics {
    sent: u64,
    ok: u64,
    err: u64,
    hist: Histogram<u64>,     // microseconds
    codes: BTreeMap<u16, u64> // status code -> count (0 == transport error)
}

impl Metrics {
    /// Creates a new `Metrics` instance with default values.
    fn new() -> Self {
        let mut h = Histogram::<u64>::new_with_bounds(1, 10_000_000, 3).unwrap(); // 1us..10s
        h.auto(true);
        Self {
            sent: 0,
            ok: 0,
            err: 0,
            hist: h,
            codes: BTreeMap::new(),
        }
    }

    /// Records the result of a request.
    /// 
    /// # Arguments
    /// 
    /// * `ok` - Whether the request was successful.
    /// * `lat_us` - Latency of the request in microseconds.
    /// * `code` - Optional HTTP status code.
    fn record(&mut self, ok: bool, lat_us: u64, code: Option<u16>) {
        self.sent += 1;
        if ok {
            self.ok += 1;
        } else {
            self.err += 1;
        }
        let _ = self.hist.record(lat_us.min(10_000_000));
        let key = code.unwrap_or(0);
        *self.codes.entry(key).or_insert(0) += 1;
    }
}

/// Entry point for the load tester application.
/// 
/// This function initializes logging, parses command-line arguments, loads the configuration,
/// and either spawns worker processes or runs as a single worker.
/// 
/// # Errors
/// 
/// Returns an error if the configuration is invalid or if any worker process fails.
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();
    let cfg = load_config(&cli.config)?;

    if cfg.processes == 0 {
        return Err(anyhow!("processes must be >= 1"));
    }

    match cli.worker {
        None => {
            if cfg.processes == 1 {
                run_worker(0, &cfg).await?;
            } else {
                spawn_workers(&cli, &cfg)?;
                // Master also participates as worker 0
                run_worker(0, &cfg).await?;
            }
        }
        Some(id) => run_worker(id, &cfg).await?,
    }

    Ok(())
}

/// Loads the configuration file.
/// 
/// This function reads the configuration file from the specified path and deserializes it into a `Config` struct.
/// 
/// # Arguments
/// 
/// * `path` - Path to the configuration file.
/// 
/// # Errors
/// 
/// Returns an error if the file cannot be read or if the contents are invalid.
fn load_config(path: &PathBuf) -> Result<Config> {
    let data = fs::read_to_string(path).with_context(|| format!("reading {:?}", path))?;
    // TOML if extension says so; else try YAML then TOML
    if path
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| e.eq_ignore_ascii_case("toml"))
        .unwrap_or(false)
    {
        Ok(toml::from_str::<Config>(&data).context("parsing TOML config")?)
    } else {
        match serde_yaml::from_str::<Config>(&data) {
            Ok(c) => Ok(c),
            Err(yerr) => toml::from_str::<Config>(&data)
                .map_err(|terr| anyhow!("YAML error: {yerr}\nTOML error: {terr}")),
        }
    }
}

/// Spawns worker processes for the load test.
/// 
/// This function creates additional OS processes to distribute the load testing work.
/// 
/// # Arguments
/// 
/// * `cli` - Command-line arguments.
/// * `cfg` - Configuration options.
/// 
/// # Errors
/// 
/// Returns an error if any worker process fails to spawn.
fn spawn_workers(cli: &Cli, cfg: &Config) -> Result<()> {
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

/// Determines if an HTTP status code indicates success.
/// 
/// # Arguments
/// 
/// * `code` - HTTP status code.
/// 
/// # Returns
/// 
/// `true` if the status code is in the 2xx range; otherwise, `false`.
#[inline]
fn is_ok_status(code: u16) -> bool { code / 100 == 2 }

/// Runs a worker process for the load test.
/// 
/// This function performs the actual load testing work, sending HTTP requests and collecting metrics.
/// 
/// # Arguments
/// 
/// * `worker_id` - ID of the worker process.
/// * `cfg` - Configuration options.
/// 
/// # Errors
/// 
/// Returns an error if any request fails or if the configuration is invalid.
async fn run_worker(worker_id: usize, cfg: &Config) -> Result<()> {
    info!("Worker {worker_id} starting…");

    let duration = humantime::parse_duration(&cfg.duration)
        .with_context(|| format!("invalid duration: {}", cfg.duration))?;
    let timeout = humantime::parse_duration(&cfg.timeout)
        .with_context(|| format!("invalid timeout: {}", cfg.timeout))?;

    let client = Client::builder()
        .timeout(timeout)
        .danger_accept_invalid_certs(!cfg.verify_tls)
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .build()?;

    let method = match cfg.method.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        other => return Err(anyhow!("unsupported method: {other}")),
    };

    let payload = if let Some(path) = &cfg.payload_file {
        Some(fs::read(path).with_context(|| format!("reading payload file {path}"))?)
    } else {
        None
    };

    // Split global rate across processes
    let base = cfg.rate_per_sec / cfg.processes as u32;
    let rem = cfg.rate_per_sec % cfg.processes as u32;
    let my_rate = base + if worker_id < rem as usize { 1 } else { 0 };
    if my_rate == 0 {
        info!("Worker {worker_id}: assigned rate is 0 req/s; exiting.");
        return Ok(());
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
        let url = cfg.target_url.clone();
        let headers = cfg.headers.clone();
        let method = method.clone();
        let payload = payload.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            let _permit = permit; // keeps semaphore slot until drop
            let req_start = Instant::now();

            let mut req = client.request(method, &url);
            for (k, v) in headers {
                if let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
                    req = req.header(name, v);
                }
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

    let elapsed_s = start_at.elapsed().as_secs_f64().max(1e-6);
    print_summary(worker_id, &metrics, elapsed_s);

    Ok(())
}

/// Prints a summary of the metrics collected by a worker.
/// 
/// # Arguments
/// 
/// * `worker_id` - ID of the worker process.
/// * `m` - Metrics collected during the load test.
/// * `elapsed_s` - Total elapsed time in seconds.
fn print_summary(worker_id: usize, m: &Metrics, elapsed_s: f64) {
    let rps = m.sent as f64 / elapsed_s;
    println!(
        concat!(
            "[worker {worker}] sent={sent} ok={ok} err={err} ",
            "rps={rps:.1} ",
            "p50={p50:.2}ms p95={p95:.2}ms p99={p99:.2}ms"
        ),
        worker = worker_id,
        sent = m.sent,
        ok = m.ok,
        err = m.err,
        rps = rps,
        p50 = ms(p(&m.hist, 50.0)),
        p95 = ms(p(&m.hist, 95.0)),
        p99 = ms(p(&m.hist, 99.0)),
    );

    if !m.codes.is_empty() {
        println!("status codes:");
        for (code, count) in &m.codes {
            if *code == 0 {
                println!("  0 (transport errors): {}", count);
            } else {
                println!("  {}: {}", code, count);
            }
        }
    }
}

/// Retrieves the value at a specific quantile from a histogram.
/// 
/// # Arguments
/// 
/// * `hist` - Histogram containing latency data.
/// * `q` - Quantile (e.g., 50.0 for p50).
/// 
/// # Returns
/// 
/// The value at the specified quantile, or 0 if the histogram is empty.
fn p(hist: &Histogram<u64>, q: f64) -> u64 {
    if hist.len() == 0 {
        return 0;
    }
    hist.value_at_quantile(q / 100.0)
}

/// Converts microseconds to milliseconds.
/// 
/// # Arguments
/// 
/// * `us` - Time in microseconds.
/// 
/// # Returns
/// 
/// Time in milliseconds as a floating-point number.
fn ms(us: u64) -> f64 {
    us as f64 / 1000.0
}