use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub target_url: String,
    pub method: String, // "GET" or "POST"
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub payload_file: Option<String>,
    /// e.g. "30s"
    pub duration: String,
    /// total rate; master splits across workers
    pub rate_per_sec: u32,
    /// async concurrency per process
    pub concurrency_per_process: usize,
    /// number of OS processes
    pub processes: usize,
    /// per-request timeout, e.g. "5s"
    pub timeout: String,
    /// Max idle connection per host on the pool.
    pub pool_max_idle: usize,
    /// How long idle connections are kept before being closed.
    pub pool_idle_timeout: u64,
    #[serde(default = "default_verify_tls")]
    pub verify_tls: bool,
}

fn default_verify_tls() -> bool { true }

pub fn load_config(path: &PathBuf) -> Result<Config> {
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

pub fn print_config(cfg: &Config) {
    println!("Configuration:");
    println!("  Target URL: {}", cfg.target_url);
    println!("  Method: {}", cfg.method);
    println!("  Headers: {:?}", cfg.headers);
    match &cfg.payload_file {
        Some(p) => println!("  Payload File: {}", p),
        None => println!("  Payload File: None"),
    }
    println!("  Duration: {}", cfg.duration);
    println!("  Rate per Second: {}", cfg.rate_per_sec);
    println!("  Concurrency per Process: {}", cfg.concurrency_per_process);
    println!("  Processes: {}", cfg.processes);
    println!("  Timeout: {}", cfg.timeout);
    println!("  Max idle conn per pool: {}", cfg.pool_max_idle);
    println!("  Idle connections keep time out: {}", cfg.pool_idle_timeout);
    println!("  Verify TLS: {}", cfg.verify_tls);
}