use rstress_core::config::Config;

#[test]
fn parse_yaml_minimal() {
    let yaml = r#"
target_url: "https://example.com"
method: "GET"
duration: "1s"
rate_per_sec: 1
concurrency_per_process: 1
processes: 1
timeout: "1s"
"#;
    let cfg: Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.target_url, "https://example.com");
    assert_eq!(cfg.method, "GET");
}

#[test]
fn parse_toml_minimal() {
    let toml = r#"
target_url = "https://example.com"
method = "POST"
duration = "1s"
rate_per_sec = 10
concurrency_per_process = 2
processes = 1
timeout = "1s"
"#;
    let cfg: Config = toml::from_str(toml).unwrap();
    assert_eq!(cfg.method, "POST");
}
