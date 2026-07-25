use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;

use crate::models::Rule;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct RuleInput {
    pub name:        Option<String>,
    pub local_port:  i32,
    pub remote_host: String,
    pub remote_port: i32,
    pub protocol:    Option<String>,
    pub enabled:     Option<bool>,
    pub log_enabled: Option<bool>,
    pub log_body:    Option<bool>,
}

fn validate(input: &RuleInput) -> Option<String> {
    if !(1..=65535).contains(&input.local_port) {
        return Some(format!("local_port {} out of range", input.local_port));
    }
    if !(1..=65535).contains(&input.remote_port) {
        return Some(format!("remote_port {} out of range", input.remote_port));
    }
    if input.remote_host.trim().is_empty() {
        return Some("remote_host is required".to_string());
    }
    None
}

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_as::<_, Rule>("SELECT * FROM rules ORDER BY local_port")
        .fetch_all(&state.db)
        .await
    {
        Ok(rules) => Json(rules).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<RuleInput>,
) -> impl IntoResponse {
    if let Some(msg) = validate(&input) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response();
    }

    let name        = input.name.unwrap_or_default();
    let protocol    = input.protocol.unwrap_or_else(|| "auto".to_string());
    let enabled     = input.enabled.unwrap_or(true);
    let log_enabled = input.log_enabled.unwrap_or(true);
    let log_body    = input.log_body.unwrap_or(false);

    let result = sqlx::query_as::<_, Rule>(
        r#"INSERT INTO rules (name, local_port, remote_host, remote_port, protocol, enabled, log_enabled, log_body)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)
           RETURNING *"#,
    )
    .bind(&name)
    .bind(input.local_port)
    .bind(&input.remote_host)
    .bind(input.remote_port)
    .bind(&protocol)
    .bind(enabled)
    .bind(log_enabled)
    .bind(log_body)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(rule) => {
            if rule.enabled {
                if let Err(e) = state.manager.start(rule.clone()).await {
                    let msg = e.to_string();
                    if msg.contains("address already in use") || msg.contains("already in use") {
                        return (
                            StatusCode::CONFLICT,
                            Json(json!({"error": format!("port already in use: {}", rule.local_port)})),
                        )
                            .into_response();
                    }
                    tracing::warn!("failed to start listener: {msg}");
                }
            }
            (StatusCode::CREATED, Json(rule)).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.to_uppercase().contains("UNIQUE") {
                return (
                    StatusCode::CONFLICT,
                    Json(json!({"error": format!("port already in use: {}", input.local_port)})),
                )
                    .into_response();
            }
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": msg}))).into_response()
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<RuleInput>,
) -> impl IntoResponse {
    // Fetch existing.
    let existing = match sqlx::query_as::<_, Rule>("SELECT * FROM rules WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response(),
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
        }
    };

    if let Some(msg) = validate(&input) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response();
    }

    let name        = input.name.unwrap_or(existing.name);
    let protocol    = input.protocol.unwrap_or(existing.protocol);
    let enabled     = input.enabled.unwrap_or(existing.enabled);
    let log_enabled = input.log_enabled.unwrap_or(existing.log_enabled);
    let log_body    = input.log_body.unwrap_or(existing.log_body);
    let updated_at  = Utc::now();

    let result = sqlx::query_as::<_, Rule>(
        r#"UPDATE rules SET name=?, local_port=?, remote_host=?, remote_port=?,
           protocol=?, enabled=?, log_enabled=?, log_body=?, updated_at=?
           WHERE id=? RETURNING *"#,
    )
    .bind(&name)
    .bind(input.local_port)
    .bind(&input.remote_host)
    .bind(input.remote_port)
    .bind(&protocol)
    .bind(enabled)
    .bind(log_enabled)
    .bind(log_body)
    .bind(updated_at)
    .bind(id)
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(rule)) => {
            if rule.enabled {
                if let Err(e) = state.manager.restart(rule.clone()).await {
                    let msg = e.to_string();
                    if msg.contains("address already in use") || msg.contains("already in use") {
                        return (
                            StatusCode::CONFLICT,
                            Json(json!({"error": format!("port already in use: {}", rule.local_port)})),
                        )
                            .into_response();
                    }
                    tracing::warn!("failed to restart listener: {msg}");
                }
            } else {
                state.manager.stop(rule.local_port).await.ok();
            }
            Json(rule).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.to_uppercase().contains("UNIQUE") {
                return (
                    StatusCode::CONFLICT,
                    Json(json!({"error": format!("port already in use: {}", input.local_port)})),
                )
                    .into_response();
            }
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": msg}))).into_response()
        }
    }
}

pub async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    // Fetch the rule first to get the local_port for stopping.
    let rule = match sqlx::query_as::<_, Rule>("SELECT * FROM rules WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response(),
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
        }
    };

    state.manager.stop(rule.local_port).await.ok();

    let res = sqlx::query("DELETE FROM rules WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn toggle(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let rule = match sqlx::query_as::<_, Rule>("SELECT * FROM rules WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response(),
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
        }
    };

    let new_enabled = !rule.enabled;
    let updated_at  = Utc::now();

    let updated = match sqlx::query_as::<_, Rule>(
        "UPDATE rules SET enabled=?, updated_at=? WHERE id=? RETURNING *",
    )
    .bind(new_enabled)
    .bind(updated_at)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response(),
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
        }
    };

    if new_enabled {
        if let Err(e) = state.manager.start(updated.clone()).await {
            tracing::warn!("failed to start after toggle: {e}");
        }
    } else {
        state.manager.stop(updated.local_port).await.ok();
    }

    Json(updated).into_response()
}
