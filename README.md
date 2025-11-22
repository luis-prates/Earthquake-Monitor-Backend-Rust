# Earthquake Monitor — Backend (Rust / Axum)

A small backend service that ingests earthquake data from the USGS feed, stores events in PostgreSQL, and exposes a REST API with OpenAPI docs and basic monitoring.

This repository is intentionally minimal but structured for incremental improvements and learning.

## Step 1: Run through Docker

Requirements: Docker + Docker Compose, Rust toolchain (optional for building locally)

1. copy environment variables
```bash
cp .env.example .env
# edit .env as needed (POSTGRES_DATABASE_URL is templated in .env.example)
```

2. Start services
```bash
docker compose up --build -d
```

3. Apply DB migration (only needed first time)
```bash
# run migration inside postgres container (migrations are mounted into /migrations)
docker compose exec db psql -U "${POSTGRES_USER:-postgres}" -d "${POSTGRES_DB:-earthquakes}" -f /migrations/0001_create_earthquakes.sql
```

4. Check service
- Health: http://localhost:8000/health
- API root: http://localhost:8000/earthquakes
- Swagger UI: http://localhost:8000/docs
- OpenAPI JSON: http://localhost:8000/api-doc/openapi.json
- Metrics: http://localhost:8000/metrics

5. Logs
```bash
# runtime logs (stdout)
docker compose logs -f app

# file logs (inside container)
docker compose exec app tail -n 200 /var/log/earthquake/app.log
```

# Run locally (with Rust)

If you prefer running locally without Docker:

1. Ensure POSTGRES_DATABASE_URL points to a reachable Postgres instance (e.g. postgresql://postgres:postgres@localhost:5432/earthquakes)

2. Run migration (psql or sqlx migrate if configured)

3. Build and run

```bash
cargo build --release
# run the server
POSTGRES_DATABASE_URL=... RUST_LOG=info ./target/release/earthquake-monitor-backend-rust
```

# Tests

Run unit & integration tests:
```bash
cargo test -- --nocapture
```

There are lightweight, hermetic integration tests that exercise the router and OpenAPI generation (no DB).

# API

Main endpoints:

- GET /health — health check
- GET /earthquakes — list earthquakes with optional query filters:
  - min_magnitude (float)
  - max_magnitude (float)
  - start_time (ISO8601 UTC)
  - end_time (ISO8601 UTC)
  - limit (int, default 50, max 500)
  - offset (int, default 0)
- GET /earthquakes/{id} — fetch by UUID
- GET /metrics — Prometheus metrics
- GET /docs — Swagger UI
- GET /api-doc/openapi.json — OpenAPI JSON

Responses use a ListResponse wrapper:
```json
{
  "data": [ ... ],
  "pagination": { "limit": 50, "offset": 0, "total": 123 }
}
```

# Migration / Schema

Migration file: `migrations/0001_create_earthquakes.sql`

Core schema (summary):
- `id UUID PRIMARY KEY`
- `usgs_id TEXT UNIQUE` — original USGS id (used for dedupe)
- `location TEXT`
- `magnitude REAL`
- `depth_km REAL`
- `latitude REAL`, `longitude REAL`
- `time TIMESTAMP WITH TIME ZONE`

Indexes:
- `idx_earthquakes_time ON time DESC`
- `idx_earthquakes_magnitude ON magnitude DESC`
- `idx_earthquakes_usgs_id ON usgs_id`

Apply migration using psql (see Quick Start).

# Design decisions & notes

See design_decisions.md for the full rationale.

# Assumptions & limitations

See assumptions_and_limitations.md.

## CI / Tests

This repository includes a GitHub Actions workflow (`.github/workflows/ci.yml`) that runs on pushes and PRs to `main`. The workflow:

- checks code formatting (`cargo fmt`),
- runs `cargo clippy` with `-D warnings`,
- runs the full test suite (`cargo test`),
- builds the Docker image (locally in the runner).

Check the Actions tab in GitHub for build logs and test output.

## Container healthcheck

The Docker image includes a `HEALTHCHECK` that periodically probes `GET /health`. Orchestrators (Docker, Kubernetes) can use the container health status to perform restarts or route traffic only to healthy pods.

