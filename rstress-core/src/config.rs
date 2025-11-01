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

    pub duration: String,                // "30s"
    pub rate_per_sec: u32,               // total rate across processes
    pub concurrency_per_process: usize,  // per process
    pub processes: usize,                // OS processes
    pub timeout: String,                 // per-request timeout
    #[serde(default = "default_verify_tls")]
    pub verify_tls: bool,

    // NEW: connection pool knobs
    #[serde(default = "default_pool_max_idle")]
    pub pool_max_idle: usize,
    #[serde(default = "default_pool_idle_timeout")]
    pub pool_idle_timeout: u64, // seconds
}

fn default_verify_tls() -> bool { true }
fn default_pool_max_idle() -> usize { usize::MAX } // allow reuse by default
fn default_pool_idle_timeout() -> u64 { 30 }

pub fn load_config(path: &PathBuf) -> Result<Config> {
    let data = fs::read_to_string(path).with_context(|| format!("reading {:?}", path))?;
    if path.extension().and_then(|s| s.to_str()).map(|e| e.eq_ignore_ascii_case("toml")).unwrap_or(false) {
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
    println!("  Payload File: {}", cfg.payload_file.as_deref().unwrap_or("None"));
    println!("  Duration: {}", cfg.duration);
    println!("  Rate per Second: {}", cfg.rate_per_sec);
    println!("  Concurrency per Process: {}", cfg.concurrency_per_process);
    println!("  Processes: {}", cfg.processes);
    println!("  Timeout: {}", cfg.timeout);
    println!("  Verify TLS: {}", cfg.verify_tls);
    println!("  Pool Max Idle/Host: {}", cfg.pool_max_idle);
    println!("  Pool Idle Timeout: {}s", cfg.pool_idle_timeout);
}