use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct PlaceSummary {
    pub id: i64,
    pub name: String,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub place_type: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct StaffRow {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub phone: Option<String>,
    pub national_id: String,
    pub address: Option<String>,
    pub photo_path: Option<String>,
    pub gender: String,
    pub role: String,
    pub is_seasonal: i64,
    pub season_start_date: Option<String>,
    pub season_end_date: Option<String>,
    pub is_active: i64,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Staff {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub phone: Option<String>,
    pub national_id: String,
    pub address: Option<String>,
    pub photo_path: Option<String>,
    pub photo_url: Option<String>,
    pub gender: String,
    pub role: String,
    pub is_seasonal: bool,
    pub season_start_date: Option<String>,
    pub season_end_date: Option<String>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub main_places: Vec<PlaceSummary>,
    pub rest_days: Vec<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaffInput {
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Option<String>,
    pub phone: Option<String>,
    pub national_id: String,
    pub address: Option<String>,
    pub gender: String,
    pub role: String,
    pub is_seasonal: bool,
    pub season_start_date: Option<String>,
    pub season_end_date: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub photo_source_path: Option<String>,
    #[serde(default)]
    pub remove_photo: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Absence {
    pub id: i64,
    pub staff_id: i64,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub absence_type: String,
    pub start_date: String,
    pub end_date: String,
    pub reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AbsenceInput {
    pub staff_id: i64,
    #[serde(rename = "type")]
    pub absence_type: String,
    pub start_date: String,
    pub end_date: String,
    pub reason: Option<String>,
}
