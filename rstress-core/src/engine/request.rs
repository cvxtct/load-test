use anyhow::Result;
use reqwest::{header::HeaderName, Method};

use crate::config::Config;

#[derive(Clone)]
pub struct RequestSpec {
    pub url: String,
    pub method: Method,
    pub headers: Vec<(HeaderName, String)>,
    pub payload: Option<Vec<u8>>,
}

pub fn build_request_spec(cfg: &Config) -> Result<RequestSpec> {
    let method = match cfg.method.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        other => anyhow::bail!("unsupported method: {other}"),
    };

    let headers = cfg
        .headers
        .iter()
        .filter_map(|(k, v)| {
            HeaderName::from_bytes(k.as_bytes())
                .ok()
                .map(|hn| (hn, v.clone()))
        })
        .collect::<Vec<_>>();

    let payload = if let Some(path) = &cfg.payload_file {
        Some(std::fs::read(path)?)
    } else {
        None
    };

    Ok(RequestSpec {
        url: cfg.target_url.clone(),
        method,
        headers,
        payload,
    })
}
