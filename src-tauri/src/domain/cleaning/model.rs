use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::domain::staff::model::Staff;

#[derive(Debug, Clone, FromRow)]
pub struct CleaningServiceRow {
    pub id: i64,
    pub place_id: i64,
    pub place_name: String,
    pub date: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub service_type: String,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ServiceStaffRow {
    pub service_id: i64,
    pub staff_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub photo_path: Option<String>,
    pub gender: String,
    pub is_lead: i64,
    pub is_active: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStaff {
    pub staff_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub photo_url: Option<String>,
    pub gender: String,
    pub is_lead: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleaningService {
    pub id: i64,
    pub place_id: i64,
    pub place_name: String,
    pub date: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub service_type: String,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub staff: Vec<ServiceStaff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStaffInput {
    pub staff_id: i64,
    pub is_lead: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CleaningServiceInput {
    pub place_id: i64,
    pub date: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub service_type: String,
    pub status: String,
    pub notes: Option<String>,
    pub staff: Vec<ServiceStaffInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffSuggestion {
    pub staff: Staff,
    pub reasons: Vec<String>,
}
