use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct IncidentRow {
    pub id: i64,
    pub service_id: i64,
    pub place_id: i64,
    pub place_name: String,
    pub service_type: String,
    #[sqlx(rename = "type")]
    pub incident_type: String,
    pub description: String,
    pub photo_path: Option<String>,
    pub status: String,
    pub date: String,
    pub resolved_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Incident {
    pub id: i64,
    pub service_id: i64,
    pub place_id: i64,
    pub place_name: String,
    pub service_type: String,
    #[serde(rename = "type")]
    pub incident_type: String,
    pub description: String,
    pub photo_path: Option<String>,
    pub photo_url: Option<String>,
    pub status: String,
    pub date: String,
    pub resolved_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncidentInput {
    pub service_id: i64,
    #[serde(rename = "type")]
    pub incident_type: String,
    pub description: String,
    pub date: String,
    #[serde(default)]
    pub photo_source_path: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct IncidentListFilters {
    pub status: Option<String>,
    #[serde(alias = "type")]
    pub incident_type: Option<String>,
    pub date: Option<String>,
    pub service_id: Option<i64>,
}
