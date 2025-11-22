# Earthquake Service (Rust + Axum)

This repo contains a simple backend service to ingest and serve earthquake data.

## Step 1: Run locally (dev)

Requirements: Docker + Docker Compose, Rust toolchain (optional for building locally)

Start Postgres + app:
```bash
docker compose up --build

## CI / Tests

This repository includes a GitHub Actions workflow (`.github/workflows/ci.yml`) that runs on pushes and PRs to `main`. The workflow:

- checks code formatting (`cargo fmt`),
- runs `cargo clippy` with `-D warnings`,
- runs the full test suite (`cargo test`),
- builds the Docker image (locally in the runner).

Check the Actions tab in GitHub for build logs and test output.

## Container healthcheck

The Docker image includes a `HEALTHCHECK` that periodically probes `GET /health`. Orchestrators (Docker, Kubernetes) can use the container health status to perform restarts or route traffic only to healthy pods.

