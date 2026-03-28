use zerocopy::{FromBytes, Immutable, IntoBytes};

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, Copy, Clone)]
pub struct IpcHeader {
    pub version: u8,         // Protocol version (e.g., 0x01)
    pub execution_mode: u8,  // 0x00 (Sync) or 0x01 (Async/TTC)
    pub command_id: u16,     // DAG workflow ID
    pub trace_id: u32,       // Request mapping ID
    pub payload_length: u32, // Length of the following raw context
    pub _padding: [u8; 4],   // 4-byte padding for 8-byte alignment safety
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_header_layout() {
        assert_eq!(
            std::mem::size_of::<IpcHeader>(),
            16,
            "IpcHeader must be exactly 16 bytes per C-ABI contract"
        );
        // Ensure no hidden padding altered the packed structure behavior
        assert_eq!(std::mem::align_of::<IpcHeader>(), 4);
    }
}
