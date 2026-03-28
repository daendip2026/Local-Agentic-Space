mod ipc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Headless Local Agentic Kernel Daemon");

    ipc::start_ipc_server().await
}
