use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

pub struct TcpResult {
    pub bytes_client: u64,
    pub bytes_server: u64,
    pub preview:      Option<String>,
}

pub async fn handle_tcp(
    mut client: TcpStream,
    remote_addr: &str,
    log_body: bool,
) -> Result<TcpResult> {
    // Dial remote with 10-second timeout.
    let mut remote = timeout(
        Duration::from_secs(10),
        TcpStream::connect(remote_addr),
    )
    .await
    .map_err(|_| anyhow!("dial timeout connecting to {remote_addr}"))?
    .map_err(|e| anyhow!("dial error: {e}"))?;

    let mut bytes_client: u64 = 0;
    let mut bytes_server: u64 = 0;
    let mut preview: Option<String> = None;

    // If log_body: capture the first chunk from client before entering bidirectional copy.
    if log_body {
        let mut preview_buf = vec![0u8; 4096];
        match client.read(&mut preview_buf).await {
            Ok(0) => {
                // Client closed immediately — nothing to forward.
                return Ok(TcpResult { bytes_client: 0, bytes_server: 0, preview: None });
            }
            Ok(n) => {
                bytes_client += n as u64;
                // Check if the captured bytes are valid UTF-8.
                if let Ok(s) = std::str::from_utf8(&preview_buf[..n]) {
                    preview = Some(s.to_string());
                }
                // Write the peeked bytes to remote before entering copy_bidirectional.
                remote.write_all(&preview_buf[..n]).await?;
            }
            Err(e) => {
                tracing::debug!("tcp read preview error: {e}");
                return Err(anyhow!("preview read error: {e}"));
            }
        }
    }

    // Bidirectional copy for the rest of the connection.
    match tokio::io::copy_bidirectional(&mut client, &mut remote).await {
        Ok((c2s, s2c)) => {
            bytes_client += c2s;
            bytes_server += s2c;
        }
        Err(e) => {
            // Connection-reset / broken-pipe are normal — log at debug and return what we have.
            tracing::debug!("tcp copy_bidirectional ended: {e}");
        }
    }

    Ok(TcpResult { bytes_client, bytes_server, preview })
}
