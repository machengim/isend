use anyhow::{anyhow, Result};
use async_std::net::{UdpSocket, TcpStream};
use log::{debug, info};
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use super::arg::SendArg;
use super::instruction::{Instruction, Operation};
use super::message::{Message, send_msg};
use super::utils;

lazy_static::lazy_static! {
    static ref BLACK_LIST: Mutex<Vec<SocketAddr>> = Mutex::new(Vec::new());
}

// Entry function of Sender.
// Bind on a UDP socket, listen incoming UDP connection,
// get the target TCP port.
pub async fn launch(arg: SendArg) -> Result<()> {    
    let udp = UdpSocket::bind(("0.0.0.0", 0)).await?;
    let port = udp.local_addr()?.port();
    //println!("Connection code: {}", port);
    send_msg(Message::Status(format!("Connection code: {}", port)));
    listen_udp(&udp, arg.expire, arg.password.as_ref()).await?;

    Ok(())
}

// Listen UDP socket, until a connection comes with a valid port number,
// assume it's the TCP port of the receiver.
// TODO: return stream after successful connection.
async fn listen_udp(udp: &UdpSocket, expire: u8, password: Option<&String>) -> Result<TcpStream>{
    let mut buf = [0; 2];
    let start = Instant::now();

    loop {
        if start.elapsed().as_secs() > (expire * 60) as u64 { break; }

        let (_, addr) = udp.recv_from(&mut buf).await?;
        let port = u16::from_be_bytes(buf);
        let socket = SocketAddr::new(addr.ip(), port);

        // If this socket already in black list, ignore it.
        if BLACK_LIST.lock().unwrap().contains(&socket) {
            debug!("Found socket in black list {}", &socket);
            continue;
        }

        debug!("Connection request from {}", socket);
        if let Some(stream) = try_connect_tcp(&socket, password).await? {
            return Ok(stream);
        }

        async_std::task::sleep(Duration::from_secs(1)).await;
    }

    Err(anyhow!("No connection established in time"))
}

// Try to connect to the target machine after receiving its connection request.
// Only run once for a connection request.
// Needs reply from receiver to continue next step.
// TODO: return stream.
async fn try_connect_tcp(socket: &SocketAddr, password: Option<&String>) 
    -> Result<Option<TcpStream>> {
    
    let mut stream = TcpStream::connect(socket).await?;
    utils::send_ins(&mut stream, 0, Operation::Connect, password).await?;

    let response = utils::recv_ins(&mut stream).await?;
    match response.operation {
        Operation::ConnSuccess => {
            info!("Connection established");
            return Ok(Some(stream));
        },
        Operation::ConnRefuse => {
            let detail = utils::recv_content(&mut stream, response.length as usize).await?;
            //println!("Connection refused: {}", String::from_utf8(detail)?);
            send_msg(Message::Error(format!("Connection refused: {}", String::from_utf8(detail)?)));
            // Add this socket to black list if connection being refused to avoid future attemp.
            BLACK_LIST.lock().unwrap().push(*socket);
        },
        Operation::ConnError => { 
            let detail = utils::recv_content(&mut stream, response.length as usize).await?;
            //println!("Error when connecting: {}", String::from_utf8(detail)?);
            send_msg(Message::Error(format!("in connection: {}", String::from_utf8(detail)?)))
        },
        _ =>  println!("Unknow instruction in reponse"),
    }

    Ok(None)
}