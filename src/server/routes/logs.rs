use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::models::RequestLog;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub rule_id:   Option<i64>,
    pub page:      Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ClearQuery {
    pub rule_id: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<LogQuery>,
) -> impl IntoResponse {
    let page      = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(50).min(200).max(1);
    let offset    = ((page - 1) * page_size) as i64;
    let limit     = page_size as i64;

    let (total, rows) = if let Some(rule_id) = params.rule_id {
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM request_logs WHERE rule_id = ?")
                .bind(rule_id)
                .fetch_one(&state.db)
                .await
                .unwrap_or(0);
        let rows = sqlx::query_as::<_, RequestLog>(
            "SELECT * FROM request_logs WHERE rule_id = ? ORDER BY id DESC LIMIT ? OFFSET ?",
        )
        .bind(rule_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        (total, rows)
    } else {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM request_logs")
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);
        let rows = sqlx::query_as::<_, RequestLog>(
            "SELECT * FROM request_logs ORDER BY id DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        (total, rows)
    };

    Json(json!({"total": total, "items": rows})).into_response()
}

pub async fn clear(
    State(state): State<AppState>,
    Query(params): Query<ClearQuery>,
) -> impl IntoResponse {
    let result = if let Some(rule_id) = params.rule_id {
        sqlx::query("DELETE FROM request_logs WHERE rule_id = ?")
            .bind(rule_id)
            .execute(&state.db)
            .await
    } else {
        sqlx::query("DELETE FROM request_logs")
            .execute(&state.db)
            .await
    };

    match result {
        Ok(r) => Json(json!({"deleted": r.rows_affected()})).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
