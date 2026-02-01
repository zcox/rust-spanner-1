use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Response type for successful PUT operations
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct PutResponse {
    pub id: String,
}

/// Response type for successful GET operations
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct GetResponse {
    pub id: String,
    pub data: JsonValue,
}

/// Query parameters for list endpoint
#[derive(Deserialize, utoipa::ToSchema)]
pub struct ListQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub prefix: Option<String>,
    pub sort: Option<String>,
}

/// Response type for list endpoint
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct ListResponse {
    pub data: Vec<KvEntryResponse>,
    pub total_count: i64,
}

/// Individual key-value entry in list response
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct KvEntryResponse {
    pub key: String,
    pub value: JsonValue,
    pub created_at: String,
    pub updated_at: String,
}
