use crate::db::DbPool;
use axum::{Json, extract::Path, extract::State, http::StatusCode};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn health() -> &'static str {
    "OK"
}

pub async fn list_earthquakes(
    State(pool): State<DbPool>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Placeholder: return a static sample for step 1
    let sample = json!([
        {
            "id": "00000000-0000-0000-0000-000000000000",
            "location": "Sampleville",
            "magnitude": 4.5,
            "latitude": 34.05,
            "longitude": -118.25,
            "depth_km": 10.0,
            "time": "2025-11-20T00:00:00Z"
        }
    ]);
    Ok(Json(sample))
}

pub async fn get_earthquake(
    Path(id): Path<String>,
    State(_pool): State<DbPool>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Placeholder: return 404 if sample id mismatches
    if id == "00000000-0000-0000-0000-000000000000" {
        let sample = json!({
            "id": id,
            "location": "Sampleville",
            "magnitude": 4.5,
            "latitude": 34.05,
            "longitude": -118.25,
            "depth_km": 10.0,
            "time": "2025-11-20T00:00:00Z"
        });
        Ok(Json(sample))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            format!("earthquake {} not found", id),
        ))
    }
}
