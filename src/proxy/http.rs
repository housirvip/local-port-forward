use anyhow::Result;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, HeaderMap, Request, Response, StatusCode};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use hyper_util::rt::{TokioExecutor, TokioIo};
use serde_json::{Map, Value};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

use crate::models::RequestLog;

/// Drive a single accepted TCP connection through all its HTTP/1.1 request/response cycles.
pub async fn handle_http_conn(
    conn: TcpStream,
    target_base: Arc<String>,
    log_body: bool,
    log_tx: broadcast::Sender<RequestLog>,
    rule_id: i64,
    src_addr: String,
    db: SqlitePool,
) {
    let io = TokioIo::new(conn);

    let svc = service_fn(move |req: Request<Incoming>| {
        let target_base = target_base.clone();
        let log_tx = log_tx.clone();
        let src_addr = src_addr.clone();
        let db = db.clone();
        async move { forward(req, target_base, log_body, log_tx, rule_id, src_addr, db).await }
    });

    if let Err(e) = http1::Builder::new()
        .keep_alive(true)
        .serve_connection(io, svc)
        .await
    {
        tracing::debug!("http1 serve_connection ended: {e}");
    }
}

async fn forward(
    req: Request<Incoming>,
    target_base: Arc<String>,
    log_body: bool,
    log_tx: broadcast::Sender<RequestLog>,
    rule_id: i64,
    src_addr: String,
    db: SqlitePool,
) -> Result<Response<Full<Bytes>>, std::convert::Infallible> {
    let start = Instant::now();

    let method = req.method().clone();
    let path = req
        .uri()
        .path_and_query()
        .map(|p| p.to_string())
        .unwrap_or_else(|| "/".to_string());

    // Clone headers before body is consumed.
    let req_headers: HeaderMap = req.headers().clone();
    let req_headers_json = headers_to_json(&req_headers);

    // Target host for Host header (strip scheme).
    let target_host = target_base
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .to_string();

    // Collect request body.
    let body_bytes = match req.into_body().collect().await {
        Ok(c) => c.to_bytes(),
        Err(_) => Bytes::new(),
    };
    let req_body_bytes_len = body_bytes.len() as i64;
    let req_body_str = if log_body {
        let capped = body_bytes.slice(..body_bytes.len().min(65536));
        Some(String::from_utf8_lossy(&capped).to_string())
    } else {
        None
    };

    // Build upstream URI.
    let upstream_uri_str = format!("{}{}", target_base, path);
    let upstream_uri = match upstream_uri_str.parse::<hyper::Uri>() {
        Ok(u) => u,
        Err(e) => {
            tracing::debug!("bad upstream uri {upstream_uri_str}: {e}");
            return Ok(error_response(400, "Bad Request"));
        }
    };

    // Build upstream request, forwarding all original headers.
    let mut upstream_builder = hyper::Request::builder()
        .method(method.clone())
        .uri(upstream_uri);

    // Copy original headers, override Host, append X-Forwarded-For.
    for (k, v) in &req_headers {
        if k == hyper::header::HOST {
            continue; // will set below
        }
        upstream_builder = upstream_builder.header(k, v);
    }
    upstream_builder = upstream_builder.header(hyper::header::HOST, &target_host);
    upstream_builder = upstream_builder.header("X-Forwarded-For", &src_addr);

    let upstream_req = upstream_builder
        .body(Full::new(body_bytes))
        .unwrap();

    // Execute upstream request.
    let client: Client<HttpConnector, Full<Bytes>> =
        Client::builder(TokioExecutor::new()).build(HttpConnector::new());

    let (http_status, resp_headers_json, resp_body_str, resp_body_bytes, resp_headers) =
        match client.request(upstream_req).await {
            Ok(resp) => {
                let status = resp.status().as_u16() as i32;
                let resp_hdrs = resp.headers().clone();
                let resp_hdr_json = headers_to_json(&resp_hdrs);
                let resp_bytes = resp
                    .into_body()
                    .collect()
                    .await
                    .map(|c| c.to_bytes())
                    .unwrap_or_default();
                let resp_body = if log_body {
                    let capped = resp_bytes.slice(..resp_bytes.len().min(65536));
                    Some(String::from_utf8_lossy(&capped).to_string())
                } else {
                    None
                };
                (status, resp_hdr_json, resp_body, resp_bytes, resp_hdrs)
            }
            Err(e) => {
                tracing::debug!("upstream request failed: {e}");
                (502, "{}".to_string(), None, Bytes::from("Bad Gateway"), HeaderMap::new())
            }
        };

    let duration_ms = start.elapsed().as_millis() as i64;
    let bytes_transferred = req_body_bytes_len + resp_body_bytes.len() as i64;

    // Insert log row and broadcast.
    let log_entry = insert_log(
        &db,
        rule_id,
        &src_addr,
        "http",
        Some(&method.to_string()),
        Some(&path),
        Some(http_status),
        Some(&req_headers_json),
        req_body_str.as_deref(),
        Some(&resp_headers_json),
        resp_body_str.as_deref(),
        None,
        bytes_transferred,
        duration_ms,
    )
    .await;

    if let Ok(entry) = log_entry {
        log_tx.send(entry).ok();
    }

    // Build client response, forwarding upstream response headers.
    let status = StatusCode::from_u16(http_status as u16).unwrap_or(StatusCode::BAD_GATEWAY);
    let mut response = Response::builder().status(status);
    for (k, v) in &resp_headers {
        // Skip connection-level headers that hyper manages.
        if k == hyper::header::CONNECTION || k == hyper::header::TRANSFER_ENCODING {
            continue;
        }
        response = response.header(k, v);
    }
    Ok(response.body(Full::new(resp_body_bytes)).unwrap())
}

fn error_response(status: u16, msg: &'static str) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .body(Full::new(Bytes::from(msg)))
        .unwrap()
}

fn headers_to_json(headers: &HeaderMap) -> String {
    let mut map = Map::new();
    for (k, v) in headers.iter() {
        if let Ok(val) = v.to_str() {
            map.insert(k.as_str().to_string(), Value::String(val.to_string()));
        }
    }
    serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
}

#[allow(clippy::too_many_arguments)]
async fn insert_log(
    db: &SqlitePool,
    rule_id: i64,
    src_addr: &str,
    protocol: &str,
    http_method: Option<&str>,
    http_path: Option<&str>,
    http_status: Option<i32>,
    http_req_headers: Option<&str>,
    http_req_body: Option<&str>,
    http_resp_headers: Option<&str>,
    http_resp_body: Option<&str>,
    tcp_preview: Option<&str>,
    bytes_transferred: i64,
    duration_ms: i64,
) -> Result<RequestLog> {
    let row = sqlx::query_as::<_, RequestLog>(
        r#"INSERT INTO request_logs
           (rule_id, src_addr, protocol, http_method, http_path, http_status,
            http_req_headers, http_req_body, http_resp_headers, http_resp_body,
            tcp_preview, bytes_transferred, duration_ms)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           RETURNING *"#,
    )
    .bind(rule_id)
    .bind(src_addr)
    .bind(protocol)
    .bind(http_method)
    .bind(http_path)
    .bind(http_status)
    .bind(http_req_headers)
    .bind(http_req_body)
    .bind(http_resp_headers)
    .bind(http_resp_body)
    .bind(tcp_preview)
    .bind(bytes_transferred)
    .bind(duration_ms)
    .fetch_one(db)
    .await?;
    Ok(row)
}
