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

#[cfg(test)]
mod tests {
    use super::*;
    use agentic_ipc::IpcHeader;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;
    use zerocopy::{FromBytes, IntoBytes};

    #[tokio::test]
    async fn test_ipc_echo() {
        let socket_path = "/tmp/agentic-kernel-test.sock";
        let _ = std::fs::remove_file(socket_path);
        let listener = UnixListener::bind(socket_path).unwrap();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream).await.unwrap();
        });

        // Brief yield to ensure the listener is ready
        tokio::task::yield_now().await;

        let mut client = UnixStream::connect(socket_path).await.unwrap();

        let header = IpcHeader {
            version: 1,
            execution_mode: 0,
            command_id: 10,
            trace_id: 1234,
            payload_length: 5,
            _padding: Default::default(),
        };

        client.write_all(header.as_bytes()).await.unwrap();
        client.write_all(b"HELLO").await.unwrap();

        let mut resp_header_buf = [0u8; 16];
        client.read_exact(&mut resp_header_buf).await.unwrap();
        let resp_header = IpcHeader::read_from_bytes(&resp_header_buf[..]).unwrap();

        assert_eq!(resp_header.version, 1);
        assert_eq!(resp_header.trace_id, 1234);

        let mut resp_payload = vec![0u8; resp_header.payload_length as usize];
        client.read_exact(&mut resp_payload).await.unwrap();
        assert_eq!(&resp_payload, b"HELLO");

        // Cleanup
        let _ = std::fs::remove_file(socket_path);
    }
}
