use anyhow::Result;
use async_std::net::TcpListener;
use super::arg::RecvArg;

pub async fn launch(r: RecvArg) -> Result<()> {
    let tcp_socket = TcpListener::bind(("0.0.0.0", 0)).await?;
    let tcp_port = tcp_socket.local_addr()?.port();

    Ok(())
}
