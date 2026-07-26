use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;

use crate::models::Rule;
use crate::server::AppState;

#[derive(Debug, serde::Serialize)]
pub struct RuleView {
    #[serde(flatten)]
    pub rule: Rule,
    pub bind_error: Option<String>,
}

fn to_view(state: &AppState, rule: Rule) -> RuleView {
    let bind_error = if rule.enabled {
        state.manager.bind_error(rule.local_port)
    } else {
        None
    };
    RuleView { rule, bind_error }
}

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

fn validate(input: &RuleInput) -> Result<(), String> {
    if !(1..=65535).contains(&input.local_port) {
        return Err(format!("local_port {} out of range", input.local_port));
    }
    if (1..1024).contains(&input.local_port) {
        return Err(format!(
            "Port {} is a privileged port (< 1024). It may require root/admin privileges.",
            input.local_port
        ));
    }
    if !(1..=65535).contains(&input.remote_port) {
        return Err(format!("remote_port {} out of range", input.remote_port));
    }
    if input.remote_host.trim().is_empty() {
        return Err("remote_host is required".to_string());
    }
    Ok(())
}

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_as::<_, Rule>("SELECT * FROM rules ORDER BY local_port")
        .fetch_all(&state.db)
        .await
    {
        Ok(rules) => Json(rules.into_iter().map(|r| to_view(&state, r)).collect::<Vec<_>>()).into_response(),
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
    if let Err(msg) = validate(&input) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response();
    }

    let name        = input.name.unwrap_or_default();
    let protocol    = input.protocol.unwrap_or_else(|| "auto".to_string());
    let enabled     = input.enabled.unwrap_or(true);
    let log_enabled = input.log_enabled.unwrap_or(true);
    let log_body    = input.log_body.unwrap_or(false);

    let listener = if enabled {
        let addr = format!("0.0.0.0:{}", input.local_port);
        match TcpListener::bind(&addr).await {
            Ok(l) => Some(l),
            Err(e) => {
                let msg = e.to_string();
                let hint = if msg.contains("Permission denied") || msg.contains("permission denied") {
                    format!("Port {} requires elevated privileges. Try a port above 1024.", input.local_port)
                } else if input.local_port < 1024 {
                    format!("Port {} is a privileged port, please modify settings or run as root.", input.local_port)
                } else {
                    format!("Port {} is already in use: {}", input.local_port, msg)
                };
                return (StatusCode::CONFLICT, Json(json!({"error": hint}))).into_response();
            }
        }
    } else {
        None
    };

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
            if let Some(listener) = listener {
                if let Err(e) = state.manager.start_with_listener(rule.clone(), listener).await {
                    tracing::warn!("failed to start listener: {e}");
                }
            }
            (StatusCode::CREATED, Json(to_view(&state, rule))).into_response()
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

    if let Err(msg) = validate(&input) {
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
            if existing.local_port != rule.local_port {
                // Port changed: the old port's listener is unrelated to the new
                // one restart()/stop() below manage, so it must be stopped explicitly
                // or it keeps running and silently forwarding to the stale target.
                state.manager.stop(existing.local_port).await.ok();
            }
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
            Json(to_view(&state, rule)).into_response()
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
    let result = sqlx::query_as::<_, Rule>("DELETE FROM rules WHERE id = ? RETURNING *")
        .bind(id)
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(rule)) => {
            state.manager.stop(rule.local_port).await.ok();
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response(),
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

    Json(to_view(&state, updated)).into_response()
}
