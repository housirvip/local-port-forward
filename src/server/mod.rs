pub mod routes;

use std::sync::Arc;

use axum::{
    http::{header, StatusCode, Uri},
    response::Response,
    routing::{get, post, put},
    Router,
};
use rust_embed::RustEmbed;
use sqlx::SqlitePool;
use tokio::sync::broadcast;

use crate::models::RequestLog;
use crate::proxy::Manager;

#[derive(RustEmbed)]
#[folder = "web/"]
struct WebAssets;

#[derive(Clone)]
pub struct AppState {
    pub db:          SqlitePool,
    pub manager:     Arc<Manager>,
    pub log_tx:      broadcast::Sender<RequestLog>,
    pub listen_addr: String,
    pub db_path:     String,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/rules",            get(routes::rules::list).post(routes::rules::create))
        .route("/api/rules/:id",        put(routes::rules::update).delete(routes::rules::delete_rule))
        .route("/api/rules/:id/toggle", post(routes::rules::toggle))
        .route("/api/logs",             get(routes::logs::list).delete(routes::logs::clear))
        .route("/api/logs/stream",      get(routes::stream::stream_logs))
        .route("/api/settings", get(routes::settings::get_settings).put(routes::settings::update_settings))
        .fallback(serve_static)
        .with_state(state)
}

async fn serve_static(uri: Uri) -> Response {
    let raw = uri.path().trim_start_matches('/');
    let path = if raw.is_empty() { "index.html" } else { raw };

    match WebAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .body(axum::body::Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => {
            // SPA fallback — serve index.html for any unrecognised path.
            let idx = WebAssets::get("index.html").unwrap();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(axum::body::Body::from(idx.data.into_owned()))
                .unwrap()
        }
    }
}
