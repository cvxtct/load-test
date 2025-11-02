# `rstress` web application stress tester in Rust
![status](https://img.shields.io/badge/status-experimental-orange)
![license](https://img.shields.io/badge/license-MIT-green)
![rust](https://img.shields.io/badge/rust-🦀-blue)


`rstress` is a HTTP load testing tool written in Rust. The endeavor is to keep it fast, simple, and reliable. — The capability to ***expose the breaking point of the system with precision.***

This project started as a study and experimentation tool to explore async concurrency, multi-process coordination, in Rust.


## Key Features

- Built on tokio and reqwest for high concurrency without complexity
- Precise latency histograms using hdrhistogram — see what really happens under pressure
- Multi-process mode to push CPU and network limits cleanly
- Metrics that matter: RPS, quantiles, status codes, and transport errors
- Realistic client behavior — tune timeouts, connection reuse, and idle socket lifetimes to see how the system handles real-world load
- Structured output — generate JSON reports with configuration and results for tracking runs over time



## Project Status

This is an experimental and evolving project — use at your own risk.
APIs, output formats, and internal structures may change between versions.
That said, it’s stable enough to run real-world load experiments, measure latency, and stress endpoints safely in controlled environments.

## Motivation

I’ve always wanted a tailored tool that I can spin up within moments — one that provides clear, meaningful insights into application and system performance over time.

`rstress` is both a practical tool and a learning journey, a personal playground to explore async concurrency, performance measurement, and Rust itself. 

## Planned Improvements

- Payload providers – unified trait.
- Dynamic traffic shaping – smoothly transition from normal to extreme loads.
- Auth layer – pluggable authentication (Bearer, Basic, custom headers).
- Response handling – optional validation for status codes or body regex matches.
- Per-worker configuration – fine-tuned setups per process.
- Adaptive rate control – dynamically adjust tick interval based on measured RPS to stay on target.
- Per-request timeout – finer-grained control with tokio::time::timeout.
- HTTP/2 vs HTTP/1.1 – expose protocol selection in config to study parallelism effects.
- DNS strategy – allow fixed IPs or pre-resolved targets to remove DNS noise.
- Payload reuse – leverage Bytes + Arc for efficient reuse of large static bodies.


## Usage


```bash
git clone
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

## Report
---
```
[worker 0] sent=91 ok=91 err=0 rps=3.0 p50=552.45ms p95=552.96ms p99=553.47ms
status codes:
  200: 91
[worker 1] sent=61 ok=61 err=0 rps=2.0 p50=551.93ms p95=552.96ms p99=553.47ms
status codes:
  200: 61
```

