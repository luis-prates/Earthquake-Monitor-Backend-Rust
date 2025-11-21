use crate::db::DbPool;
use crate::models::{Earthquake, ListResponse, Pagination};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

pub async fn health() -> &'static str {
    "OK"
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListParams {
    pub min_magnitude: Option<f32>,
    pub max_magnitude: Option<f32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[utoipa::path(
    get,
    path = "/earthquakes",
    params(
        ("min_magnitude" = Option<f32>, Query, description = "Minimum magnitude to filter"),
        ("max_magnitude" = Option<f32>, Query, description = "Maximum magnitude to filter"),
        ("start_time" = Option<String>, Query, description = "ISO8601 start time"),
        ("end_time" = Option<String>, Query, description = "ISO8601 end time"),
        ("limit" = Option<i64>, Query, description = "Max items to return"),
        ("offset" = Option<i64>, Query, description = "Result offset"),
    ),
    responses(
        (status = 200, description = "List earthquakes", body = ListResponse<Earthquake>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Database error")
    )
)]
pub async fn list_earthquakes(
    State(pool): State<DbPool>,
    Query(params): Query<ListParams>,
) -> Result<Json<ListResponse<Earthquake>>, (StatusCode, String)> {
    // Build base query and dynamic filters
    let mut query = "SELECT id, usgs_id, location, magnitude, depth_km, latitude, longitude, time FROM earthquakes".to_string();
    let mut conditions: Vec<String> = Vec::new();
    // let mut args: Vec<sqlx::types::Json<serde_json::Value>> = Vec::new();
    // We'll build with simple string concatenation and bind params in order.
    // For clarity and safety, use positional parameter indexes ($1, $2, ...).
    let mut binds: Vec<String> = Vec::new();
    let mut idx = 1;

    if let Some(min_mag) = params.min_magnitude {
        conditions.push(format!("magnitude >= ${}", idx));
        binds.push(min_mag.to_string());
        idx += 1;
    }

    if let Some(max_mag) = params.max_magnitude {
        conditions.push(format!("magnitude <= ${}", idx));
        binds.push(max_mag.to_string());
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
    for b in &binds {
        // Our binds are strings; we attempt to bind as appropriate types
        // Try to detect if it was a float vs datetime string
        if let Ok(f) = b.parse::<f32>() {
            sqlx_q = sqlx_q.bind(f);
        } else if let Ok(dt) = DateTime::parse_from_rfc3339(b) {
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

    // total count with same filters (without limit/offset)
    let mut count_sql = "SELECT count(*) as total FROM earthquakes".to_string();
    if !conditions.is_empty() {
        count_sql.push_str(" WHERE ");
        count_sql.push_str(&conditions.join(" AND "));
    }

    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &binds {
        if let Ok(f) = b.parse::<f32>() {
            count_query = count_query.bind(f);
        } else if let Ok(dt) = DateTime::parse_from_rfc3339(b) {
            let dt_utc: DateTime<Utc> = dt.with_timezone(&Utc);
            count_query = count_query.bind(dt_utc);
        } else if let Ok(i) = b.parse::<i64>() {
            count_query = count_query.bind(i);
        } else {
            count_query = count_query.bind(b);
        }
    }

    let total = count_query.fetch_one(&pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db error: {}", e),
        )
    })?;

    let response = ListResponse {
        data: rows,
        pagination: Pagination {
            limit,
            offset,
            total,
        },
    };

    Ok(Json(response))
}

/// GET /earthquakes/{id}
/// Fetch a specific earthquake by UUID.
#[utoipa::path(
    get,
    path = "/earthquakes/{id}",
    params(
        ("id" = String, Path, description = "UUID of the earthquake")
    ),
    responses(
        (status = 200, description = "Earthquake details", body = Earthquake),
        (status = 404, description = "Not found"),
        (status = 400, description = "Invalid UUID")
    )
)]
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
