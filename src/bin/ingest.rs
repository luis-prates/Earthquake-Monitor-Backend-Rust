use anyhow::Result;
use chrono::{TimeZone, Utc};
use reqwest::Client;
use serde_json::Value;
use sqlx::PgPool;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@db:5432/earthquakes".into());
    let pool = PgPool::connect(&database_url).await?;
    let client = Client::new();
    let feed_url = env::var("USGS_FEED_URL").unwrap_or_else(|_| {
        "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_day.geojson".into()
    });
    let interval_secs: u64 = env::var("INGEST_INTERVAL_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);

    loop {
        if let Err(e) = fetch_and_store(&client, &pool, &feed_url).await {
            tracing::error!("ingest error: {}", e);
        }
        sleep(Duration::from_secs(interval_secs)).await;
    }
}

async fn fetch_and_store(client: &Client, pool: &PgPool, feed_url: &str) -> Result<()> {
    tracing::info!("fetching USGS feed: {}", feed_url);
    let resp = client.get(feed_url).send().await?.error_for_status()?;
    let body: Value = resp.json().await?;

    if let Some(features) = body.get("features").and_then(|f| f.as_array()) {
        tracing::info!("received {} features", features.len());

        for feature in features {
            if let Some(props) = feature.get("properties") {
                let usgs_id = feature
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let mag = props.get("mag").and_then(|m| m.as_f64()).unwrap_or(0.0) as f32;
                let place = props
                    .get("place")
                    .and_then(|p| p.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let time_ms = props.get("time").and_then(|t| t.as_i64()).unwrap_or(0);
                let time = Utc
                    .timestamp_opt(time_ms / 1000, ((time_ms % 1000) * 1_000_000) as u32)
                    .single()
                    .unwrap_or_else(Utc::now);
                // geometry: [longitude, latitude, depth(km)]
                let (lon, lat, depth_km) = feature
                    .get("geometry")
                    .and_then(|g| g.get("coordinates"))
                    .and_then(|coords| coords.as_array())
                    .and_then(|arr| {
                        if arr.len() >= 3 {
                            let lon = arr[0].as_f64().unwrap_or(0.0) as f32;
                            let lat = arr[1].as_f64().unwrap_or(0.0) as f32;
                            let depth_km = arr[2].as_f64().unwrap_or(0.0) as f32;
                            Some((lon, lat, depth_km))
                        } else {
                            None
                        }
                    })
                    .unwrap_or((0.0, 0.0, 0.0));

                // Insert - generate a UUID locally
                let id = Uuid::new_v4();

                let usgs_id_ref = usgs_id.clone();

                // Use ON CONFLICT DO NOTHING on usgs_id to avoid duplicates
                sqlx::query(
                    r#"
                    INSERT INTO earthquakes (id, usgs_id, location, magnitude, depth_km, latitude, longitude, time)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    ON CONFLICT (usgs_id) DO NOTHING
                    "#)
                    .bind(id)
                    .bind(usgs_id_ref)
                    .bind(place)
                    .bind(mag)
                    .bind(depth_km)
                    .bind(lat)
                    .bind(lon)
                    .bind(time)
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(())
}
