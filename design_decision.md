# Design decisions — Earthquake Monitor Backend

This file explains the key design decisions and the trade-offs made while building the project.

## Tech stack
- **Rust + Axum** — modern, async-first HTTP framework; lightweight and expressive.
- **sqlx** — direct SQL with optional compile-time checks. Favoring explicit SQL over an ORM to keep DB behavior obvious and teach SQL.
- **PostgreSQL** — robust relational DB with strong time and numeric support; easy to extend to PostGIS later.
- **reqwest (rustls)** — HTTP client to fetch USGS GeoJSON.
- **utoipa + utoipa-swagger-ui** — OpenAPI generation and Swagger UI for easy API discovery.
- **tracing + tracing-subscriber + tracing-appender** — structured logs and file appending (daily rolling).

## Schema choices
- Flat `earthquakes` table with a `usgs_id` field and a UUID primary key.
  - Rationale: USGS provides a stable ID; use it for deduplication. We still assign a UUID so internal references are canonical and DB-generated.
- Use `REAL` (float) for lat/lon/depth/magnitude:
  - Rationale: simple, compact, and sufficient precision for listings and basic filtering.
  - Trade-off: If you need geospatial queries (distance, bounding boxes), migrate to PostGIS (`geometry` types).
- `time TIMESTAMP WITH TIME ZONE`:
  - Rationale: store event times in UTC with timezone awareness.

## Indexing
- `time DESC` — optimized for "recent" queries.
- `magnitude DESC` — optimized for filtering by magnitude and retrieving highest events quickly.
- `usgs_id` unique index — deduplication.

## Ingestion approach
- Poll USGS GeoJSON feed at configurable interval (`INGEST_INTERVAL_SECONDS`).
- Parse the GeoJSON features; insert using `ON CONFLICT (usgs_id) DO NOTHING`.
  - Rationale: simple and robust. For higher data volumes, consider a message queue (Kafka) and idempotent consumer.
- The ingestion worker runs as an internal background task in the main binary (spawned with `tokio::spawn`).
  - Rationale: keeps deployment simple (single binary). For more complex setups, consider a separate ingestion service.

## API design
- RESTful endpoints (`GET /earthquakes`, `GET /earthquakes/{id}`).
- Pagination & filtering via query parameters; defaults chosen to be conservative (limit=50, max 500).
- Responses are JSON; errors are returned as standard HTTP status codes and simple JSON messages.
- OpenAPI is generated using `utoipa` macros — accurate schema and docs without manual duplication.

## Observability
- `tracing` logs to both console (stdout) and a daily-rotating file.
- Prometheus metrics (simple `ingested_total` counter) and `/metrics` endpoint.
- Docker healthcheck uses `GET /health`.

## Testing
- Unit tests for small parsing logic and DB-free router tests under `tests/`.
- Integration/end-to-end tests are intentionally not heavy — you can add a test job that spins Postgres (Docker) for full-stack verification.

## Trade-offs & shortcuts (intentional)
- **No PostGIS**: avoided for simplicity. If spatial queries are required, add PostGIS.
- **Simple SQL builder**: dynamic SQL is hand-built for clarity. For complex queries prefer `sqlx::QueryBuilder`.
- **No auth**: kept out to reduce scope; add auth for production.
- **Runtime SQL queries**: using `sqlx::query()` to avoid mandating `cargo sqlx prepare` in dev. You can enable compile-time checks in CI if you wish by wiring `DATABASE_URL` and running `cargo sqlx prepare`.

---