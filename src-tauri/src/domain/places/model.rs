use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct PlaceRow {
    pub id: i64,
    pub name: String,
    #[sqlx(rename = "type")]
    pub place_type: String,
    pub status: Option<String>,
    pub location: Option<String>,
    pub check_in_date: Option<String>,
    pub check_out_date: Option<String>,
    pub is_active: i64,
    pub cancelled_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub place_type: String,
    pub location: Option<String>,
    pub status: Option<String>,
    pub effective_status: String,
    pub recommended_service_type: Option<String>,
    pub check_in_date: Option<String>,
    pub check_out_date: Option<String>,
    pub is_active: bool,
    pub cancelled_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaceInput {
    pub name: String,
    #[serde(rename = "type")]
    pub place_type: String,
    pub status: Option<String>,
    pub location: Option<String>,
    pub check_in_date: Option<String>,
    pub check_out_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecommendedTask {
    pub effective_status: String,
    pub recommended_service_type: Option<String>,
}
