use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::models::{Settings, SettingsInput, SettingsResponse};
use crate::server::AppState;

pub async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_as::<_, Settings>("SELECT log_max_rows, log_ttl_days, default_protocol, default_log_enabled, default_log_body FROM settings WHERE id = 1")
        .fetch_one(&state.db)
        .await
    {
        Ok(s) => Json(SettingsResponse {
            log_max_rows:        s.log_max_rows,
            log_ttl_days:        s.log_ttl_days,
            default_protocol:    s.default_protocol,
            default_log_enabled: s.default_log_enabled,
            default_log_body:    s.default_log_body,
            listen_addr:         state.listen_addr.clone(),
            db_path:             state.db_path.clone(),
        }).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(input): Json<SettingsInput>,
) -> impl IntoResponse {
    // Load current
    let current = match sqlx::query_as::<_, Settings>(
        "SELECT log_max_rows, log_ttl_days, default_protocol, default_log_enabled, default_log_body FROM settings WHERE id = 1"
    ).fetch_one(&state.db).await {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let log_max_rows        = input.log_max_rows.unwrap_or(current.log_max_rows).max(0);
    let log_ttl_days        = input.log_ttl_days.unwrap_or(current.log_ttl_days).max(0);
    let default_protocol    = input.default_protocol.unwrap_or(current.default_protocol);
    let default_log_enabled = input.default_log_enabled.unwrap_or(current.default_log_enabled);
    let default_log_body    = input.default_log_body.unwrap_or(current.default_log_body);

    // Validate protocol
    if !matches!(default_protocol.as_str(), "auto" | "http" | "tcp") {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "default_protocol must be auto/http/tcp"}))).into_response();
    }

    match sqlx::query(
        "UPDATE settings SET log_max_rows=?, log_ttl_days=?, default_protocol=?, default_log_enabled=?, default_log_body=? WHERE id=1"
    )
    .bind(log_max_rows)
    .bind(log_ttl_days)
    .bind(&default_protocol)
    .bind(default_log_enabled)
    .bind(default_log_body)
    .execute(&state.db)
    .await {
        Ok(_) => Json(SettingsResponse {
            log_max_rows,
            log_ttl_days,
            default_protocol,
            default_log_enabled,
            default_log_body,
            listen_addr: state.listen_addr.clone(),
            db_path:     state.db_path.clone(),
        }).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
