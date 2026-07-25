use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RequestLog {
    pub id:                i64,
    pub rule_id:           i64,
    pub created_at:        DateTime<Utc>,
    pub protocol:          String,
    pub src_addr:          String,
    pub http_method:       Option<String>,
    pub http_path:         Option<String>,
    pub http_status:       Option<i32>,
    pub http_req_headers:  Option<String>,
    pub http_req_body:     Option<String>,
    pub http_resp_headers: Option<String>,
    pub http_resp_body:    Option<String>,
    pub tcp_preview:       Option<String>,
    pub bytes_transferred: i64,
    pub duration_ms:       i64,
}
