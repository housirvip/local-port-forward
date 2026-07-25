use anyhow::Result;
use portforward::{db, proxy::Manager, server};
use std::{env, sync::Arc};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let db_path     = env::var("DB_PATH").unwrap_or_else(|_| "portforward.db".to_string());
    let listen_addr = env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    let pool = db::open(&db_path).await?;
    let (log_tx, _) = broadcast::channel::<portforward::models::RequestLog>(256);
    let manager = Arc::new(Manager::new(pool.clone(), log_tx.clone()));
    manager.load_from_db().await?;

    // Spawn log cleanup background task (runs every hour)
    let cleanup_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            // Read settings
            let Ok(settings) = sqlx::query_as::<_, portforward::models::Settings>(
                "SELECT log_max_rows, log_ttl_days, default_protocol, default_log_enabled, default_log_body FROM settings WHERE id = 1"
            ).fetch_one(&cleanup_pool).await else { continue };

            if settings.log_max_rows > 0 {
                sqlx::query(
                    "DELETE FROM request_logs WHERE id NOT IN (SELECT id FROM request_logs ORDER BY id DESC LIMIT ?)"
                )
                .bind(settings.log_max_rows)
                .execute(&cleanup_pool)
                .await
                .ok();
            }

            if settings.log_ttl_days > 0 {
                sqlx::query(
                    "DELETE FROM request_logs WHERE created_at < datetime('now', ? || ' days')"
                )
                .bind(format!("-{}", settings.log_ttl_days))
                .execute(&cleanup_pool)
                .await
                .ok();
            }
        }
    });

    let state = server::AppState {
        db:          pool,
        manager:     manager.clone(),
        log_tx,
        listen_addr: listen_addr.clone(),
        db_path:     db_path.clone(),
    };
    let app = server::create_router(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!("listening on {listen_addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;

    manager.stop_all().await;
    Ok(())
}
