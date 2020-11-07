use anyhow::Result;
use async_std::net::UdpSocket;
use super::arg::SendArg;

// Entry function of Sender.
// Bind on a UDP socket, listen incoming UDP connection,
// get the target TCP port.
pub async fn launch(arg: SendArg) -> Result<()> {
    let udp = UdpSocket::bind(("0.0.0.0", 0)).await?;
    let port = udp.local_addr()?.port();
    println!("Connection code: {}", port);
    listen_udp(&udp).await?;

    Ok(())
}

// Listen UDP socket, until a connection comes with a valid port number,
// assume it's the TCP port of the receiver.
async fn listen_udp(udp: &UdpSocket) -> Result<()>{
    let mut buf = [0; 2];

    loop {
        let (_, addr) = udp.recv_from(&mut buf).await?;
        let port = u16::from_be_bytes(buf);
        // Test use: next 2 lines to display connection detail.
        println!("{}, {}", addr, port);
        break;
    }

    Ok(())
}