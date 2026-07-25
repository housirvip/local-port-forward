use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Settings {
    pub log_max_rows:          i64,
    pub log_ttl_days:          i64,
    pub default_protocol:      String,
    pub default_log_enabled:   bool,
    pub default_log_body:      bool,
}

/// Full settings response includes read-only runtime fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub log_max_rows:          i64,
    pub log_ttl_days:          i64,
    pub default_protocol:      String,
    pub default_log_enabled:   bool,
    pub default_log_body:      bool,
    pub listen_addr:           String,
    pub db_path:               String,
}

#[derive(Debug, Deserialize)]
pub struct SettingsInput {
    pub log_max_rows:          Option<i64>,
    pub log_ttl_days:          Option<i64>,
    pub default_protocol:      Option<String>,
    pub default_log_enabled:   Option<bool>,
    pub default_log_body:      Option<bool>,
    // listen_addr / db_path ignored on PUT
}
