pub mod http;
pub mod sniffer;
pub mod tcp;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::Instant;

use anyhow::{anyhow, Result};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::models::{RequestLog, Rule};
use self::sniffer::is_http;
use self::http::handle_http_conn;
use self::tcp::handle_tcp;

struct ListenerHandle {
    cancel: CancellationToken,
}

pub struct Manager {
    db:          SqlitePool,
    listeners:   Arc<StdMutex<HashMap<i32, ListenerHandle>>>,
    bind_errors: Arc<parking_lot::Mutex<HashMap<i32, String>>>,
    pub log_tx:  broadcast::Sender<RequestLog>,
}

impl Manager {
    pub fn new(db: SqlitePool, log_tx: broadcast::Sender<RequestLog>) -> Self {
        Manager {
            db,
            listeners: Arc::new(StdMutex::new(HashMap::new())),
            bind_errors: Arc::new(parking_lot::Mutex::new(HashMap::new())),
            log_tx,
        }
    }

    /// Load all enabled rules from the database and start a listener for each.
    pub async fn load_from_db(&self) -> Result<()> {
        let rules: Vec<Rule> =
            sqlx::query_as::<_, Rule>("SELECT * FROM rules WHERE enabled = 1")
                .fetch_all(&self.db)
                .await?;
        for rule in rules {
            if let Err(e) = self.start(rule).await {
                tracing::warn!("failed to start rule on startup: {e}");
            }
        }
        Ok(())
    }

    /// Bind a TCP listener on rule.local_port and spawn an accept loop.
    pub async fn start(&self, rule: Rule) -> Result<()> {
        let addr = format!("0.0.0.0:{}", rule.local_port);
        let local_port = rule.local_port;
        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                let err = anyhow!("bind {addr}: {e}");
                self.bind_errors.lock().insert(local_port, err.to_string());
                return Err(err);
            }
        };
        self.start_with_listener(rule, listener).await
    }

    /// Register an already-bound listener and spawn an accept loop.
    pub async fn start_with_listener(&self, rule: Rule, listener: TcpListener) -> Result<()> {
        let cancel = CancellationToken::new();
        {
            let mut map = self.listeners.lock().unwrap();
            // Stop any previous listener on this port.
            if let Some(old) = map.remove(&rule.local_port) {
                old.cancel.cancel();
            }
            map.insert(rule.local_port, ListenerHandle { cancel: cancel.clone() });
        }
        self.bind_errors.lock().remove(&rule.local_port);

        let db = self.db.clone();
        let log_tx = self.log_tx.clone();
        tokio::spawn(accept_loop(listener, rule, cancel, db, log_tx));
        Ok(())
    }

    /// Cancel the listener for local_port.
    pub async fn stop(&self, local_port: i32) -> Result<()> {
        let mut map = self.listeners.lock().unwrap();
        if let Some(handle) = map.remove(&local_port) {
            handle.cancel.cancel();
        }
        self.bind_errors.lock().remove(&local_port);
        Ok(())
    }

    /// Stop then start (used when a rule is updated).
    pub async fn restart(&self, rule: Rule) -> Result<()> {
        self.stop(rule.local_port).await?;
        self.start(rule).await
    }

    /// Cancel all running listeners (called on shutdown).
    pub async fn stop_all(&self) {
        let mut map = self.listeners.lock().unwrap();
        for (_, handle) in map.drain() {
            handle.cancel.cancel();
        }
    }

    /// Return the last bind error recorded for local_port, if any.
    pub fn bind_error(&self, local_port: i32) -> Option<String> {
        self.bind_errors.lock().get(&local_port).cloned()
    }
}

async fn accept_loop(
    listener: TcpListener,
    rule: Rule,
    cancel: CancellationToken,
    db: SqlitePool,
    log_tx: broadcast::Sender<RequestLog>,
) {
    loop {
        tokio::select! {
            res = listener.accept() => {
                match res {
                    Ok((conn, _)) => {
                        let rule = rule.clone();
                        let db = db.clone();
                        let log_tx = log_tx.clone();
                        tokio::spawn(handle_conn(conn, rule, db, log_tx));
                    }
                    Err(e) => {
                        tracing::debug!("accept error on port {}: {e}", rule.local_port);
                        break;
                    }
                }
            }
            _ = cancel.cancelled() => {
                tracing::debug!("listener on port {} cancelled", rule.local_port);
                break;
            }
        }
    }
}

async fn handle_conn(
    conn: tokio::net::TcpStream,
    rule: Rule,
    db: SqlitePool,
    log_tx: broadcast::Sender<RequestLog>,
) {
    let src_addr = match conn.peer_addr() {
        Ok(a) => a.to_string(),
        Err(_) => "unknown".to_string(),
    };
    let remote_addr = format!("{}:{}", rule.remote_host, rule.remote_port);
    let start = Instant::now();

    // Peek first 8 bytes to detect HTTP (non-consuming).
    let mut peek_buf = [0u8; 8];
    let peeked = conn.peek(&mut peek_buf).await.unwrap_or(0);

    let use_http = match rule.protocol.as_str() {
        "http" => true,
        "tcp"  => false,
        _      => is_http(&peek_buf[..peeked]),
    };

    if use_http {
        // HTTP: handle_http_conn drives the full connection lifecycle and inserts its own logs.
        handle_http_conn(
            conn,
            Arc::new(format!("http://{remote_addr}")),
            rule.log_body,
            log_tx,
            rule.id,
            src_addr,
            db,
        )
        .await;
    } else {
        // TCP: single bidirectional copy; log once when the connection closes.
        let result = handle_tcp(conn, &remote_addr, rule.log_body).await;
        let duration_ms = start.elapsed().as_millis() as i64;

        if rule.log_enabled {
            let (bytes_transferred, tcp_preview) = match &result {
                Ok(r) => (r.bytes_client as i64 + r.bytes_server as i64, r.preview.clone()),
                Err(_) => (0, None),
            };

            let log_result = sqlx::query_as::<_, RequestLog>(
                r#"INSERT INTO request_logs
                   (rule_id, src_addr, protocol, tcp_preview, bytes_transferred, duration_ms)
                   VALUES (?, ?, 'tcp', ?, ?, ?)
                   RETURNING *"#,
            )
            .bind(rule.id)
            .bind(&src_addr)
            .bind(tcp_preview.as_deref())
            .bind(bytes_transferred)
            .bind(duration_ms)
            .fetch_one(&db)
            .await;

            match log_result {
                Ok(entry) => { log_tx.send(entry).ok(); }
                Err(e) => tracing::warn!("failed to insert tcp log: {e}"),
            }
        }
    }
}
