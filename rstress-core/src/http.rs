use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

use crate::config::Config;

pub fn build_client(cfg: &Config) -> Result<Client> {
    let timeout = humantime::parse_duration(&cfg.timeout)?;
    Ok(Client::builder()
        .pool_max_idle_per_host(cfg.pool_max_idle)
        .pool_idle_timeout(Duration::from_secs(cfg.pool_idle_timeout))
        .timeout(timeout)
        .danger_accept_invalid_certs(!cfg.verify_tls)
        .build()?)
}