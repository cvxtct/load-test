use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub target_url: String,
    pub method: String, // "GET" or "POST"
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub payload_file: Option<String>,

    pub duration: String,               // "30s"
    pub rate_per_sec: u32,              // total rate across processes
    pub concurrency_per_process: usize, // per process
    pub processes: usize,               // OS processes
    pub timeout_c: String,              // per-request client level timeout
    pub timeout_r: u64,                 // per-request request level timeout
    #[serde(default = "default_verify_tls")]
    pub verify_tls: bool,

    // NEW: connection pool knobs
    #[serde(default = "default_pool_max_idle")]
    pub pool_max_idle: usize,
    #[serde(default = "default_pool_idle_timeout")]
    pub pool_idle_timeout: u64, // seconds
}

fn default_verify_tls() -> bool {
    true
}
fn default_pool_max_idle() -> usize {
    usize::MAX
} // allow reuse by default
fn default_pool_idle_timeout() -> u64 {
    30
}

pub fn load_config(path: &PathBuf) -> Result<Config> {
    let data = fs::read_to_string(path).with_context(|| format!("reading {:?}", path))?;
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
    // println!("  Headers: {:?}", cfg.headers); <--- removed until retract.
    println!(
        "  Payload File: {}",
        cfg.payload_file.as_deref().unwrap_or("None")
    );
    println!("  Duration: {}", cfg.duration);
    println!("  Rate per Second: {}", cfg.rate_per_sec);
    println!("  Concurrency per Process: {}", cfg.concurrency_per_process);
    println!("  Processes: {}", cfg.processes);
    println!("  Timeout client: {}", cfg.timeout_c);
    println!("  Timeout request: {}", cfg.timeout_r);
    println!("  Verify TLS: {}", cfg.verify_tls);
    println!("  Pool Max Idle/Host: {}", cfg.pool_max_idle);
    println!("  Pool Idle Timeout: {}s", cfg.pool_idle_timeout);
}

/* ---------------------- JSON reporting (redacted) ---------------------- */

#[derive(Debug, Clone, Serialize)]
pub struct ConfigForReport {
    pub target_url: String, // redacted query if contains tokens
    pub method: String,
    pub headers: HashMap<String, String>, // sensitive headers masked
    pub payload_file: Option<String>,
    pub duration: String,
    pub rate_per_sec: u32,
    pub concurrency_per_process: usize,
    pub processes: usize,
    pub timeout_c: String,
    pub timeout_r: u64,
    pub verify_tls: bool,
    pub pool_max_idle: usize,
    pub pool_idle_timeout: u64,
}

impl Config {
    /// Build a redacted, serializable snapshot for reports.
    pub fn redacted_for_report(&self) -> ConfigForReport {
        // 1) redact sensitive headers
        let mut headers = self.headers.clone();
        for (k, v) in headers.iter_mut() {
            match k.to_ascii_lowercase().as_str() {
                "authorization" | "proxy-authorization" | "cookie" | "x-api-key" => {
                    *v = "***redacted***".to_string();
                }
                _ => {}
            }
        }

        // 2) redact common token-like query params in target_url (best-effort, no url crate needed)
        let redacted_url = redact_tokens_in_url(&self.target_url);

        ConfigForReport {
            target_url: redacted_url,
            method: self.method.clone(),
            headers,
            payload_file: self.payload_file.clone(),
            duration: self.duration.clone(),
            rate_per_sec: self.rate_per_sec,
            concurrency_per_process: self.concurrency_per_process,
            processes: self.processes,
            timeout_c: self.timeout_c.clone(),
            timeout_r: self.timeout_r,
            verify_tls: self.verify_tls,
            pool_max_idle: self.pool_max_idle,
            pool_idle_timeout: self.pool_idle_timeout,
        }
    }
}

fn redact_tokens_in_url(u: &str) -> String {
    // very lightweight masking of query params like token, key, api_key, signature, auth, etc.
    // We avoid pulling an URL parser; this is a simple substring approach that keeps format stable.
    let Some(qpos) = u.find('?') else {
        return u.to_string();
    };
    let (_base, query) = u.split_at(qpos + 1); // include '?'
    let redacted = query
        .split('&')
        .map(|kv| {
            let mut it = kv.splitn(2, '=');
            let k = it.next().unwrap_or("");
            let v = it.next().unwrap_or("");
            let kl = k.to_ascii_lowercase();
            if matches!(
                kl.as_str(),
                "token"
                    | "access_token"
                    | "api_key"
                    | "apikey"
                    | "key"
                    | "signature"
                    | "sig"
                    | "auth"
                    | "auth_token"
            ) {
                format!("{k}=***redacted***")
            } else {
                // keep original
                if v.is_empty() {
                    k.to_string()
                } else {
                    format!("{k}={v}")
                }
            }
        })
        .collect::<Vec<_>>()
        .join("&");
    format!("{}{}", &u[..qpos + 1], redacted)
}
