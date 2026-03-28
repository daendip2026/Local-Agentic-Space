use std::io;
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

const PIPE_NAME: &str = r"\\.\pipe\agentic-kernel";

pub async fn start_ipc_server() -> io::Result<()> {
    tracing::info!("Listening on Named Pipe: {}", PIPE_NAME);

    let mut server = ServerOptions::new()
        .first_pipe_instance(true)
        .create(PIPE_NAME)?;

    loop {
        server.connect().await?;
        let connected_server = server;

        server = match ServerOptions::new().create(PIPE_NAME) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to create new pipe instance: {}", e);
                break Err(e);
            }
        };

        tokio::spawn(async move {
            if let Err(e) = handle_connection(connected_server).await {
                tracing::error!("Connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(stream: NamedPipeServer) -> io::Result<()> {
    crate::ipc::common::handle_stream(stream).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentic_ipc::IpcHeader;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::ClientOptions;
    use tokio::time::{Duration, sleep};
    use zerocopy::FromBytes;
    use zerocopy::IntoBytes;

    #[tokio::test]
    async fn test_ipc_echo() {
        tokio::spawn(async {
            let _ = start_ipc_server().await;
        });

        sleep(Duration::from_millis(100)).await;

        let mut client = ClientOptions::new().open(PIPE_NAME).unwrap();

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

        let resp_version = resp_header.version;
        let resp_trace_id = resp_header.trace_id;
        assert_eq!(resp_version, 1);
        assert_eq!(resp_trace_id, 1234);

        let mut resp_payload = vec![0u8; resp_header.payload_length as usize];
        client.read_exact(&mut resp_payload).await.unwrap();

        assert_eq!(&resp_payload, b"HELLO");
    }
}
