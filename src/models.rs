use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Earthquake {
    pub id: Uuid,
    pub usgs_id: Option<String>, // optional but helpful to dedupe USGS feed entries
    pub location: String,
    pub magnitude: f32,
    pub latitude: f32,
    pub longitude: f32,
    pub depth_km: f32,
    #[schema(value_type = Option<DateTime<Utc>>)]
    pub time: DateTime<Utc>,
}

/// Pagination metadata used in list responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}

/// Generic list response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListResponse<T>
where
    T: ToSchema + serde::Serialize,
{
    pub data: Vec<T>,
    pub pagination: Pagination,
}
