use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Rule {
    pub id:          i64,
    pub name:        String,
    pub local_port:  i32,
    pub remote_host: String,
    pub remote_port: i32,
    pub protocol:    String, // "auto" | "http" | "tcp"
    pub enabled:     bool,
    pub log_enabled: bool,
    pub log_body:    bool,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
}
