use rstress_core::{
    config::Config,
    engine::worker::run_worker,
};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use tokio::time::Duration;

#[tokio::test]
async fn worker_happy_path_get() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/ok"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let cfg = Config {
        target_url: format!("{}/ok", &server.uri()),
        method: "GET".into(),
        headers: Default::default(),
        payload_file: None,
        duration: "500ms".into(),
        rate_per_sec: 20,
        concurrency_per_process: 4,
        processes: 1,
        timeout_c: "2s".into(),
        timeout_r: 1,
        verify_tls: true,
        pool_max_idle: 10,
        pool_idle_timeout: 30,
    };

    let m = run_worker(0, &cfg).await.unwrap();
    assert!(m.sent > 0);
    assert_eq!(*m.codes.get(&200).unwrap_or(&0), m.sent);
    assert_eq!(m.err, 0);
}

#[tokio::test]
async fn worker_timeouts_are_classified() {
    let server = MockServer::start().await;

    // Respond after 1s
    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(ResponseTemplate::new(200)
            .set_delay(Duration::from_millis(1000))
            .set_body_string("slow"))
        .mount(&server)
        .await;

    let cfg = Config {
        target_url: format!("{}/slow", &server.uri()),
        method: "GET".into(),
        headers: Default::default(),
        payload_file: None,
        duration: "700ms".into(),
        rate_per_sec: 10,
        concurrency_per_process: 2,
        processes: 1,
        timeout_c: "250ms".into(),
        timeout_r: 1,
        verify_tls: true,
        pool_max_idle: 0,
        pool_idle_timeout: 1,
    };

    let m = rstress_core::engine::worker::run_worker(0, &cfg).await.unwrap();
    // Expect all to be transport timeouts (code 0) and counted under "timeout"
    assert_eq!(*m.codes.get(&0).unwrap_or(&0), m.sent);
    let to = m.transport.get("timeout").copied().unwrap_or(0);
    assert_eq!(to, m.sent);
}