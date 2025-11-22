# Assumptions and Limitations

This project is a minimal backend implemented as a learning / demo project. The sections below list the assumptions made while building it and current limitations you should be aware of.

## Assumptions

1. **USGS feed availability**
   - The service ingests earthquake data from the public USGS GeoJSON feed (configured by `USGS_FEED_URL`).
   - The feed provides:
     - `feature.id` (used as `usgs_id`),
     - `properties` including `mag`, `place`, `time`,
     - `geometry.coordinates` as `[lon, lat, depth_km]`.

2. **Single-region deployment**
   - The default deployment expects a single PostgreSQL instance reachable via `POSTGRES_DATABASE_URL`. Multi-region HA, replication, or sharding are out of scope.

3. **No authentication**
   - The service is open by default. The project assumes it will be deployed in a controlled environment or behind an API gateway.

4. **Reasonable data volume**
   - Feed sizes for `all_day` are assumed to be moderate (hundreds/thousands). The ingestion approach is polling + `ON CONFLICT DO NOTHING`. For much larger volumes or higher ingest rates, a queueing system should be introduced.

5. **Time semantics**
   - All times are stored and interpreted as UTC. Clients must provide filters in ISO8601 / UTC.

## Limitations

1. **No spatial indexing**
   - Latitude/longitude are stored as `REAL` floats. If you need distance, bounding box queries, or spatial indexing, migrate to PostGIS and use `geometry` columns and spatial indexes.

2. **Basic filter safety**
   - Filters are validated at a surface level (e.g., `min_magnitude <= max_magnitude` not strictly enforced everywhere). Clients may cause expensive DB scans with very broad queries; consider rate-limiting and more restrictive defaults for production.

3. **Scale & concurrency**
   - Pool size and database config are conservative (e.g., `max_connections = 5`). Adjust for production loads, and monitor connection saturation.

4. **Idempotency & dedupe**
   - Deduplication relies on `usgs_id`. If USGS changes ids or some features lack ids, duplicates could occur. For mission-critical dedupe, consider a stronger dedupe strategy (hashing, canonicalization).

5. **No authentication & no rate limiting**
   - Public endpoints are unauthenticated. For public production usage, add authentication, request quotas, and IP-based rate limits.

6. **Testing scope**
   - Tests are lightweight and DB-free by default. There are no full end-to-end CI tests that spin a real Postgres instance (these can be added later, e.g., using Docker Compose in CI or Testcontainers).

7. **TLS / CA certs in containers**
   - The Docker images install `ca-certificates` to avoid TLS verification failures. If you change the base image, ensure CA bundles are present.

8. **Monitoring**
   - Minimal Prometheus metric is included (`ingested_total`). Expand with histograms and more server metrics for latency, errors, and DB performance.

9. **Error handling**
   - The service surfaces reasonable errors but does not implement a full error code catalog or structured error schema. Consider adopting RFC 7807/problem+json style for production.

10. **No migrations manager**
    - Migrations are simple SQL files applied manually or via scripts. For production, adopt a migration tool (`sqlx migrate`, `refinery`, or similar) with versioning and rollback.

---

