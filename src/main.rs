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

    let state = server::AppState {
        db:      pool,
        manager: manager.clone(),
        log_tx,
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
