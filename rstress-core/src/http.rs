use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

use crate::config::Config;

pub fn build_client(cfg: &Config) -> Result<Client> {
    let timeout = humantime::parse_duration(&cfg.timeout_c)?;
    Ok(Client::builder()
        .pool_max_idle_per_host(cfg.pool_max_idle)
        .pool_idle_timeout(Duration::from_secs(cfg.pool_idle_timeout))
        .timeout(timeout)
        .danger_accept_invalid_certs(!cfg.verify_tls)
        .build()?)
}

#[cfg(test)]
mod tests {
    use super::build_client;
    use crate::config::Config;

    // Minimal valid config for building a client
    fn base_cfg() -> Config {
        Config {
            target_url: "https://example.com".to_string(),
            method: "GET".to_string(),
            headers: Default::default(),
            payload_file: None,
            duration: "1s".to_string(),
            rate_per_sec: 1,
            concurrency_per_process: 1,
            processes: 1,
            timeout_c: "1s".to_string(),
            timeout_r: 1,
            verify_tls: true,
            pool_max_idle: 10,
            pool_idle_timeout: 30,
        }
    }

    #[test]
    fn build_client_ok_with_valid_timeout() {
        let mut cfg = base_cfg();
        cfg.timeout_c = "2s".to_string();
        let client = build_client(&cfg);
        assert!(client.is_ok(), "client should build for a valid timeout");
    }

    #[test]
    fn build_client_err_with_invalid_timeout() {
        let mut cfg = base_cfg();
        cfg.timeout_c = "not-a-duration".to_string();
        let client = build_client(&cfg);
        assert!(client.is_err(), "invalid humantime duration must error");
    }

    // Optional networked tests — opt-in only
    // Enable with: cargo test -p rstress-core --features net-tests -- --ignored --nocapture
    #[cfg(feature = "net-tests")]
    mod net {
        use super::*;
        use reqwest::StatusCode;

        #[tokio::test]
        #[ignore]
        async fn verify_tls_true_rejects_self_signed() {
            let mut cfg = base_cfg();
            cfg.verify_tls = true;
            cfg.timeout = "3s".to_string();
            let client = build_client(&cfg).expect("client builds");

            let res = client.get("https://self-signed.badssl.com/").send().await;
            assert!(res.is_err(), "with verify_tls=true, self-signed should fail");
        }

        #[tokio::test]
        #[ignore]
        async fn verify_tls_false_allows_self_signed() {
            let mut cfg = base_cfg();
            cfg.verify_tls = false;
            cfg.timeout = "3s".to_string();
            let client = build_client(&cfg).expect("client builds");

            let res = client.get("https://self-signed.badssl.com/").send().await;
            let ok = res.map(|r| r.status()).ok();
            assert_eq!(ok, Some(StatusCode::OK), "with verify_tls=false, request should succeed");
        }
    }
}