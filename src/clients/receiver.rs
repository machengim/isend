use anyhow::Result;
use async_std::net::TcpListener;
use crate::{entities, utils};

pub async fn launch(arg: &entities::RecvArg) -> Result<()> {
    let code = match &arg.code{
        Some(c) => c,
        None => panic!("Unknown code input."),
    };

    let tcp_socket = TcpListener::bind(("0.0.0.0", arg.port)).await?;
    let tcp_port = tcp_socket.local_addr()?.port();
    // TODO: listen on the tcp port.

    let (udp_port, pass) = utils::decode(&code)
        .expect("Cannot parse code info");

    // TODO: send udp broadcast.
    
    Ok(())
}