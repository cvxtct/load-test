use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

use crate::config::Config;

pub fn build_client(cfg: &Config) -> Result<Client> {
    let timeout = humantime::parse_duration(&cfg.timeout)?;
    let pool_max_idle = &cfg.pool_max_idle;
    let pool_idle_timeout = &cfg.pool_idle_timeout;
    Ok(Client::builder()
        .pool_max_idle_per_host(*pool_max_idle) // Disable connection reuse
        .timeout(timeout)
        .danger_accept_invalid_certs(!cfg.verify_tls)
        .pool_idle_timeout(Duration::from_secs(*pool_idle_timeout))
        .build()?)
}