use anyhow::Result;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::errors::AppError;

pub async fn start_local_oauth_server(port: u16) -> Result<String, AppError> {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        AppError::Unexpected(format!("Failed to bind to local port {}: {}", port, e))
    })?;

    // We only want to handle a single request, so we accept one connection and return.
    let (mut socket, _) = listener.accept().await.map_err(|e| {
        AppError::Unexpected(format!("Failed to accept connection: {}", e))
    })?;

    let mut buffer = [0; 4096];
    let n = socket.read(&mut buffer).await.map_err(|e| {
        AppError::Unexpected(format!("Failed to read from socket: {}", e))
    })?;

    let request = String::from_utf8_lossy(&buffer[..n]);

    // Simple parsing of GET /callback?code=123...
    // The first line should be something like "GET /callback?code=xyz HTTP/1.1"
    let mut code = String::new();
    let mut error_msg = String::new();

    if let Some(line) = request.lines().next() {
        if line.starts_with("GET ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                let path_and_query = parts[1];
                if let Some(query_str) = path_and_query.split('?').nth(1) {
                    for param in query_str.split('&') {
                        let mut kv = param.split('=');
                        if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                            if k == "code" {
                                code = v.to_string();
                            } else if k == "error" {
                                error_msg = v.to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    let response = if !code.is_empty() {
        "HTTP/1.1 200 OK\r\n\r\n<html><body><h1>Authentication Successful!</h1><p>You can close this window and return to the app.</p><script>window.close()</script></body></html>"
    } else {
        "HTTP/1.1 400 Bad Request\r\n\r\n<html><body><h1>Authentication Failed</h1><p>Failed to retrieve authorization code.</p></body></html>"
    };

    let _ = socket.write_all(response.as_bytes()).await;
    let _ = socket.flush().await;

    if !code.is_empty() {
        Ok(code)
    } else {
        Err(AppError::Unexpected(format!("OAuth failed: {}", error_msg)))
    }
}
