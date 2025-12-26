# `rstress` web application stress tester in Rust
![status](https://img.shields.io/badge/status-experimental-orange)
![license](https://img.shields.io/badge/license-MIT-green)
![rust](https://img.shields.io/badge/rust-🦀-blue)

`rstress` is an HTTP load testing tool written in Rust. It aims to stay fast, simple, and reliable while exposing a system's breaking point with precision.

This project started as a study tool to explore async concurrency and multi-process coordination in Rust.

## Key Features

- Built on tokio and reqwest for high concurrency without complexity
- Precise latency histograms using hdrhistogram
- Multi-process mode to push CPU and network limits cleanly
- Metrics that matter: RPS, quantiles, status codes, and transport errors
- Realistic client behavior: timeouts, connection reuse, idle socket lifetimes
- Structured output — generate JSON reports with configuration and results for tracking runs over time

## Project Status

This is an experimental and evolving project. APIs, output formats, and internals may change between versions. It is stable enough for real-world load experiments in controlled environments.

## Responsible Use

Use only against systems you own or have explicit permission to test. The author assumes no liability for misuse or damage.

## Planned Improvements

- Payload providers (unified trait)
- Dynamic traffic shaping (smooth ramp and spikes)
- Auth layer (Bearer, Basic, custom headers)
- Response validation (status codes or body regex)
- Per-worker configuration
- Adaptive rate control
- HTTP/2 vs HTTP/1.1 selection
- DNS strategy (fixed IPs or pre-resolved targets)
- Payload reuse (Bytes + Arc)

## Usage

```bash
git clone <repo-url>
make all
cargo run -p rstress -- --config ./config.yaml
```

```yaml
target_url: "https://example.com/api"
method: "GET"                 # or "POST"

headers:
  Content-Type: "application/json"

payload_file: "./payload.json"

duration: "10s"                # e.g. "10s", "2m", "1h"
rate_per_sec: 5               # total desired req/s
concurrency_per_process: 30
processes: 2                   # 1 = single process only
timeout: "550ms"                  # "500ms", "2s" etc.

pool_max_idle: 0
pool_idle_timeout: 0

verify_tls: true
```

## Report

```
[worker 0] sent=91 ok=91 err=0 rps=3.0 p50=552.45ms p95=552.96ms p99=553.47ms
status codes:
  200: 91
[worker 1] sent=61 ok=61 err=0 rps=2.0 p50=551.93ms p95=552.96ms p99=553.47ms
status codes:
  200: 61
```
