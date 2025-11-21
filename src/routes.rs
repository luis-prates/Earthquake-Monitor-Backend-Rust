use crate::db::DbPool;
use crate::models::Earthquake;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use sqlx::Execute;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub async fn health() -> &'static str {
    "OK"
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub min_magnitude: Option<f32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

pub async fn list_earthquakes(
    State(pool): State<DbPool>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Earthquake>>, (StatusCode, String)> {
    // Build base query and dynamic filters
    let mut query = "SELECT id, usgs_id, location, magnitude, depth_km, latitude, longitude, time FROM earthquakes".to_string();
    let mut conditions: Vec<String> = Vec::new();
    let mut args: Vec<sqlx::types::Json<serde_json::Value>> = Vec::new();
    // We'll build with simple string concatenation and bind params in order.
    // For clarity and safety, use positional parameter indexes ($1, $2, ...).
    let mut binds: Vec<String> = Vec::new();
    let mut idx = 1;

    if let Some(min_mag) = params.min_magnitude {
        conditions.push(format!("magnitude >= ${}", idx));
        binds.push(min_mag.to_string());
        idx += 1;
    }

    if let Some(start) = params.start_time {
        conditions.push(format!("time >= ${}", idx));
        binds.push(start.to_rfc3339());
        idx += 1;
    }

    if let Some(end) = params.end_time {
        conditions.push(format!("time <= ${}", idx));
        binds.push(end.to_rfc3339());
        idx += 1;
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    // Ordering & pagination
    query.push_str(" ORDER BY time DESC");

    let limit = params.limit.unwrap_or(50).min(500);
    let offset = params.offset.unwrap_or(0);

    let q = format!("{} LIMIT {} OFFSET {}", query, limit, offset);

    // Build sqlx query and bind values. Use query_as to map into Earthquake.
    let mut sqlx_q = sqlx::query_as::<_, Earthquake>(&q);

    // Bind the values in the same order we pushed them
    for b in binds {
        // Our binds are strings; we attempt to bind as appropriate types
        // Try to detect if it was a float vs datetime string
        if let Ok(f) = b.parse::<f32>() {
            sqlx_q = sqlx_q.bind(f);
        } else if let Ok(dt) = DateTime::parse_from_rfc3339(&b) {
            // bind as chrono DateTime<Utc>
            let dt_utc: DateTime<Utc> = dt.with_timezone(&Utc);
            sqlx_q = sqlx_q.bind(dt_utc);
        } else if let Ok(i) = b.parse::<i64>() {
            sqlx_q = sqlx_q.bind(i);
        } else {
            sqlx_q = sqlx_q.bind(b);
        }
    }

    // execute
    let rows = sqlx_q.fetch_all(&pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db error: {}", e),
        )
    })?;

    Ok(Json(rows))
}

pub async fn get_earthquake(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<Earthquake>, (StatusCode, String)> {
    let uuid = id
        .parse::<Uuid>()
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid uuid".to_string()))?;
    let row = sqlx::query_as::<_, Earthquake>(
        "SELECT id, usgs_id, location, magnitude, depth_km, latitude, longitude, time FROM earthquakes WHERE id = $1"
    )
    .bind(uuid)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {}", e)))?;

    match row {
        Some(eq) => Ok(Json(eq)),
        None => Err((
            StatusCode::NOT_FOUND,
            format!("earthquake {} not found", id),
        )),
    }
}
