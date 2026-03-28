pub mod common;

#[cfg(unix)]
pub mod unix;
#[cfg(unix)]
pub use unix::start_ipc_server;

#[cfg(windows)]
pub mod windows;
#[cfg(windows)]
pub use windows::start_ipc_server;
