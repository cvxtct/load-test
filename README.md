# Rust load tester


1) quick smoke test (single process)
cargo run --release -- --config ./config.yaml

2) with multiple processes (set processes: 4 in config)
cargo run --release -- --config ./config.yaml

`config.yaml`
```Yaml
# Target endpoint and method
target_url: "https://your-endpoint"
method: "GET"                 # or "POST"

# Optional headers
headers:
  Content-Type: "application/json"
  X-Test-Run: "1"

# Optional payload file (ignored for GET)
payload_file: "./payload.json"

# How long to run the test
duration: "5s"                # e.g. "10s", "2m", "1h"

# Desired total request rate across ALL processes
rate_per_sec: 10              # total desired req/s

# Async concurrency inside each process (tokio tasks in flight)
concurrency_per_process: 50

# How many OS processes to spawn (true parallelism)
processes: 2                   # 1 = single process only

# Per-request timeout
timeout: "5s"                  # "500ms", "2s" etc.

# Whether to verify TLS certificates
verify_tls: true
```

`payload.json`
```Json
{
  "hello": "world",
  "some_value": 12345
}
```