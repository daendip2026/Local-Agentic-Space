use zerocopy::{FromBytes, Immutable, IntoBytes};

// We use `repr(C)` instead of `repr(C, packed)` to avoid unaligned field access UB.
// The current field layout (u8, u8, u16, u32, u32, [u8; 4]) produces zero implicit padding
// under C alignment rules, so `repr(C)` already guarantees exactly 16 bytes.
// This is verified by compile-time `size_of` and `offset_of` assertions below.
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

// Compile-time layout verification — these assertions fire at build time, not test time.
const _: () = {
    assert!(
        std::mem::size_of::<IpcHeader>() == 16,
        "IpcHeader must be exactly 16 bytes"
    );
    assert!(std::mem::offset_of!(IpcHeader, version) == 0);
    assert!(std::mem::offset_of!(IpcHeader, execution_mode) == 1);
    assert!(std::mem::offset_of!(IpcHeader, command_id) == 2);
    assert!(std::mem::offset_of!(IpcHeader, trace_id) == 4);
    assert!(std::mem::offset_of!(IpcHeader, payload_length) == 8);
    assert!(std::mem::offset_of!(IpcHeader, _padding) == 12);
};

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
        assert_eq!(std::mem::align_of::<IpcHeader>(), 4);
    }
}
