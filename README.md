# Rust load tester

## Start

### Prerequisites
1. Install Rust: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
2. Clone this repository and navigate to the project directory:
   ```bash
   git clone <repository-url>
   cd load-test

### Steps
1. Create the configuration files:

`config.yaml`: Define the target URL, request method, concurrency, and other parameters.
`payload.json`: (Optional) Define the payload for POST requests.

2. Build the project to download dependencies:
`cargo build --release`

3. Run the application:
`cargo run --release -- --config ./config.yaml`

>Notes
>Use the --release flag for optimized performance. For debugging, omit --release to run in debug mode.
>Logs and metrics will be displayed in the terminal.

### Example config
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

### Example payload
```Json
{
  "hello": "world",
  "some_value": 12345
}
```

# Configuration Parameters

This section explains each configuration parameter in `config.yaml` and provides examples of how they interact with each other.

## Parameters

### `target_url`
- **Description**: The URL of the target endpoint to test.
- **Example**:
  ```yaml
  target_url: "https://example.com/api"
  ```
- **Notes**: Ensure the URL is accessible and matches the method (e.g., `GET` or `POST`).

### `method`
- **Description**: The HTTP method to use for requests (e.g., `GET` or `POST`).
- **Example**:
  ```yaml
  method: "POST"
  ```
- **Interaction**: If `method` is `POST`, ensure `payload_file` is specified.

### `headers`
- **Description**: Optional HTTP headers to include in requests.
- **Example**:
  ```yaml
  headers:
    Content-Type: "application/json"
    Authorization: "Bearer token"
  ```
- **Notes**: Use this to include authentication tokens or specify content types.

### `payload_file`
- **Description**: Path to a file containing the request payload (used for `POST` requests).
- **Example**:
  ```yaml
  payload_file: "./payload.json"
  ```
- **Interaction**: Ignored if `method` is `GET`.

### `duration`
- **Description**: How long the test should run.
- **Example**:
  ```yaml
  duration: "30s"
  ```
- **Notes**: Specify in seconds (`s`), minutes (`m`), or hours (`h`).

### `rate_per_sec`
- **Description**: Total number of requests per second across all processes.
- **Example**:
  ```yaml
  rate_per_sec: 100
  ```
- **Interaction**: Distributed across `processes`. For example, if `rate_per_sec: 100` and `processes: 2`, each process will handle 50 requests per second.

### `concurrency_per_process`
- **Description**: Number of concurrent requests each process can handle.
- **Example**:
  ```yaml
  concurrency_per_process: 50
  ```
- **Interaction**: Total concurrency = `processes × concurrency_per_process`. For example, if `processes: 2` and `concurrency_per_process: 50`, total concurrency is 100.

### `processes`
- **Description**: Number of OS processes to spawn for parallelism.
- **Example**:
  ```yaml
  processes: 4
  ```
- **Interaction**: Higher values simulate multiple clients but may increase CPU usage. Ensure it matches the number of CPU cores for optimal performance.

### `timeout`
- **Description**: Maximum time to wait for a response per request.
- **Example**:
  ```yaml
  timeout: "5s"
  ```
- **Notes**: Specify in seconds (`s`) or milliseconds (`ms`).

### `verify_tls`
- **Description**: Whether to verify TLS certificates.
- **Example**:
  ```yaml
  verify_tls: true
  ```
- **Notes**: Set to `false` for self-signed certificates or testing in staging environments.

## Examples of Interaction

### Example 1: High Concurrency
```yaml
target_url: "https://example.com/api"
method: "GET"
rate_per_sec: 200
processes: 4
concurrency_per_process: 50
duration: "1m"
```
- **Explanation**: This configuration sends 200 requests per second, distributed across 4 processes. Each process handles 50 concurrent requests.

### Example 2: POST with Payload
```yaml
target_url: "https://example.com/api"
method: "POST"
headers:
  Content-Type: "application/json"
payload_file: "./payload.json"
rate_per_sec: 50
processes: 2
concurrency_per_process: 25
duration: "30s"
```
- **Explanation**: This configuration sends POST requests with a JSON payload. The total rate is 50 requests per second, distributed across 2 processes, with 25 concurrent requests per process.

### Example 3: Staging Environment
```yaml
target_url: "https://staging.example.com/api"
method: "GET"
rate_per_sec: 10
processes: 1
concurrency_per_process: 10
timeout: "2s"
verify_tls: false
```
- **Explanation**: This configuration is for a staging environment with a low request rate and disabled TLS verification.