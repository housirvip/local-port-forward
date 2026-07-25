use crate::{db, models::RequestLog, models::Settings, proxy::Manager, server};
use anyhow::Result;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::broadcast;

/// 打开数据库、装载规则、启动日志清理任务，返回 (router, manager)。
/// `listen_addr` 仅作为设置页展示值存入 AppState，不在此绑定。
pub async fn init(db_path: String, listen_addr: String) -> Result<(axum::Router, Arc<Manager>)> {
    let pool = db::open(&db_path).await?;
    let (log_tx, _) = broadcast::channel::<RequestLog>(256);
    let manager = Arc::new(Manager::new(pool.clone(), log_tx.clone()));
    manager.load_from_db().await?;
    spawn_log_cleanup(pool.clone());
    let state = server::AppState {
        db: pool,
        manager: manager.clone(),
        log_tx,
        listen_addr,
        db_path,
    };
    Ok((server::create_router(state), manager))
}

pub fn spawn_log_cleanup(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let Ok(settings) = sqlx::query_as::<_, Settings>(
                "SELECT log_max_rows, log_ttl_days, default_protocol, default_log_enabled, default_log_body FROM settings WHERE id = 1",
            )
            .fetch_one(&pool)
            .await
            else {
                continue;
            };

            if settings.log_max_rows > 0 {
                sqlx::query(
                    "DELETE FROM request_logs WHERE id NOT IN (SELECT id FROM request_logs ORDER BY id DESC LIMIT ?)",
                )
                .bind(settings.log_max_rows)
                .execute(&pool)
                .await
                .ok();
            }

            if settings.log_ttl_days > 0 {
                sqlx::query(
                    "DELETE FROM request_logs WHERE created_at < datetime('now', ? || ' days')",
                )
                .bind(format!("-{}", settings.log_ttl_days))
                .execute(&pool)
                .await
                .ok();
            }
        }
    });
}
