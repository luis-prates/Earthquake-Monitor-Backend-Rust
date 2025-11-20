use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Earthquake {
    pub id: Uuid,
    pub location: String,
    pub magnitude: f32,
    pub latitude: f32,
    pub longitude: f32,
    pub depth_km: f32,
    pub time: DateTime<Utc>,
}
