#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use portforward::proxy::Manager;
use tauri::{Manager as _, RunEvent, WebviewUrl, WebviewWindowBuilder};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir
                .join("portforward.db")
                .to_string_lossy()
                .into_owned();

            let (addr, manager) = tauri::async_runtime::block_on(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
                let addr = listener.local_addr()?;
                let (router, manager) =
                    portforward::bootstrap::init(db_path, addr.to_string()).await?;
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = axum::serve(listener, router).await {
                        tracing::error!("embedded server error: {e}");
                    }
                });
                Ok::<_, anyhow::Error>((addr, manager))
            })?;

            app.manage(manager);

            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(format!("http://{addr}").parse()?),
            )
            .title("PortForward")
            .inner_size(1200.0, 800.0)
            .build()?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building tauri app")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                let manager = app.state::<Arc<Manager>>();
                tauri::async_runtime::block_on(manager.stop_all());
            }
        });
}
