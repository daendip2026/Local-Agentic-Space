use agentic_ipc::IpcHeader;
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use zerocopy::FromBytes;

const MAX_PAYLOAD_SIZE: u32 = 10 * 1024 * 1024; // 10MB limit

pub async fn handle_stream<S>(mut stream: S) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut header_buf = [0u8; 16];
    stream.read_exact(&mut header_buf).await?;

    let header = match IpcHeader::read_from_bytes(&header_buf[..]) {
        Ok(h) => h,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid header format",
            ));
        }
    };

    let payload_len = header.payload_length;
    if payload_len > MAX_PAYLOAD_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Payload too large",
        ));
    }

    // 1. Echo the header back first
    stream.write_all(&header_buf).await?;

    // 2. Read and echo the payload in chunks using only a static stack buffer (Zero-Allocation)
    let mut chunk_buf = [0u8; 1024];
    let mut remaining = payload_len as usize;

    while remaining > 0 {
        let to_read = std::cmp::min(remaining, chunk_buf.len());
        stream.read_exact(&mut chunk_buf[..to_read]).await?;
        stream.write_all(&chunk_buf[..to_read]).await?;
        remaining -= to_read;
    }

    Ok(())
}
