use std::io;
use tokio::net::{UnixListener, UnixStream};

pub async fn start_ipc_server() -> io::Result<()> {
    let socket_path = "/tmp/agentic-kernel.sock";
    let _ = std::fs::remove_file(socket_path); // ignore error if missing
    let listener = UnixListener::bind(socket_path)?;
    tracing::info!("Listening on Unix Domain Socket: {}", socket_path);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream).await {
                        tracing::error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => tracing::error!("Accept failed: {}", e),
        }
    }
}

async fn handle_connection(stream: UnixStream) -> io::Result<()> {
    crate::ipc::common::handle_stream(stream).await
}
