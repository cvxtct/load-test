# `rstress` Rust load tester 
![status](https://img.shields.io/badge/status-experimental-orange)
![license](https://img.shields.io/badge/license-MIT-green)
![rust](https://img.shields.io/badge/rust-🦀-blue)
 

### Usage
----

```bash
make all
cargo run -p rstress -- --config ./config.yaml
```


```Yaml

# Target endpoint and method
target_url: "https://example.com/api"
method: "GET"                 # or "POST"

# Optional headers
headers:
  Content-Type: "application/json"

# Optional payload file (ignored for GET)
payload_file: "./payload.json"

# Test duration.
duration: "10s"                # e.g. "10s", "2m", "1h"

# Desired total request rate across ALL processes
rate_per_sec: 5              # total desired req/s

# Async concurrency inside each process.
# Total concurrency = processes × concurrency_per_process
# 2 x 50 = 100 requests can be active at the same time across all processes.
concurrency_per_process: 30

# Num of OS processes to spawn (true parallelism)
processes: 2                   # 1 = single process only

# Per-request timeout
# The timeout is applied from when the request starts connecting 
# until the response body has finished.
timeout: "550ms"                  # "500ms", "2s" etc.

# Sets the maximum idle connection per host allowed in the pool.
# 0: Every request opens a new TCP connection.
# Puts pressure on the application.
# 10: Useful for high concurrency.
pool_max_idle: 0

# How long idle connections are kept before being closed.
pool_idle_timeout: 0

verify_tls: true
```

### Report
---
```
[worker 0] sent=91 ok=91 err=0 rps=3.0 p50=552.45ms p95=552.96ms p99=553.47ms
status codes:
  200: 91
[worker 1] sent=61 ok=61 err=0 rps=2.0 p50=551.93ms p95=552.96ms p99=553.47ms
status codes:
  200: 61
```

