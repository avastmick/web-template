# Load Testing with k6

This directory contains load testing scripts using [k6](https://k6.io/).

## Prerequisites

Install k6:

```bash
# macOS
brew install k6

# Linux (Debian/Ubuntu)
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# Docker
docker pull grafana/k6
```

## Test Scripts

### auth-load.js
Tests authentication endpoints:
- `POST /api/auth/register` - User registration
- `POST /api/auth/login` - User login

**Thresholds:**
- Login P95 < 200ms
- Register P95 < 300ms
- Error rate < 0.1%

### chat-load.js
Tests AI chat endpoints:
- `POST /api/ai/chat` - Basic chat completion
- `POST /api/ai/chat/contextual` - Contextual chat with history

**Thresholds:**
- Chat P95 < 2000ms
- Contextual chat P95 < 2500ms
- Error rate < 0.1%

## Configuration Profiles

### Smoke Test (`config/smoke.json`)
Quick sanity check with minimal load.
- 1 VU for 30 seconds
- Relaxed thresholds

### Load Test (`config/load.json`)
Standard load testing profile.
- Ramps up to 20 VUs over 5 minutes
- Tests normal production load levels

### Stress Test (`config/stress.json`)
High load stress testing.
- Ramps up to 100 VUs over 12 minutes
- Tests system limits and breaking points
- Relaxed thresholds to find failure modes

## Running Tests

### Basic Usage

```bash
# Smoke test - quick sanity check
k6 run --config config/smoke.json auth-load.js

# Load test - standard load
k6 run --config config/load.json auth-load.js

# Stress test - find breaking points
k6 run --config config/stress.json auth-load.js
```

### With Custom Settings

```bash
# Override target URL
k6 run --config config/load.json -e BASE_URL=http://staging.example.com:8081 auth-load.js

# Override VUs and duration
k6 run --vus 50 --duration 5m auth-load.js

# For chat tests with custom AI model
k6 run --config config/load.json -e AI_MODEL=gpt-4 chat-load.js
```

### Docker Usage

```bash
docker run --rm -i grafana/k6 run - < auth-load.js

# With config
docker run --rm -v $(pwd):/scripts grafana/k6 run --config /scripts/config/load.json /scripts/auth-load.js
```

## Output Options

```bash
# JSON output for analysis
k6 run --out json=results.json auth-load.js

# InfluxDB for Grafana dashboards
k6 run --out influxdb=http://localhost:8086/k6 auth-load.js

# Cloud execution (requires k6 Cloud account)
k6 cloud auth-load.js
```

## Interpreting Results

Key metrics to watch:
- `http_req_duration` - Response time percentiles
- `http_req_failed` - Request failure rate
- `errors` - Custom error rate metric
- `vus` - Current virtual users
- `iterations` - Completed test iterations

## CI/CD Integration

Add to GitHub Actions:

```yaml
- name: Run k6 load tests
  uses: grafana/k6-action@v0.3.1
  with:
    filename: server/load-tests/auth-load.js
    flags: --config server/load-tests/config/smoke.json
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BASE_URL` | `http://localhost:8081` | Server base URL |
| `AI_MODEL` | `gpt-3.5-turbo` | AI model for chat tests |
| `K6_VUS` | Varies | Number of virtual users |
