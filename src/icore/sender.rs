use anyhow::{anyhow, Result};
use async_std::net::{UdpSocket, TcpStream};
use log::{debug, info};
use std::net::SocketAddr;
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};
use super::arg::SendArg;
use super::instruction::Operation;
use super::message::{Message, self};
use super::utils;

// Store refused sockets into a black list.
lazy_static::lazy_static! {
    static ref BLACK_LIST: Mutex<Vec<SocketAddr>> = Mutex::new(Vec::new());
}

// Entry function of Sender.
// Bind on a UDP socket, listen incoming UDP connection,
// get the target TCP port.
// After expire time the whole process will be terminated.
pub async fn launch(arg: SendArg) -> Result<()> {    
    let udp = UdpSocket::bind(("0.0.0.0", 0)).await?;
    let port = udp.local_addr()?.port();
    message::send_msg(Message::Status(format!("Connection code: {}", port)));

    // Start timer.
    let (tx, rx) = mpsc::channel();
    let expire = arg.expire.clone();
    async_std::task::spawn(async move {
        timer(expire, rx).await;
    });

    // Stop timer after getting stream.
    let password = arg.password.clone();
    let mut stream = listen_udp(&udp, expire, password.as_ref()).await?;
    tx.send(true)?;

    // Start sending files and messages.
    start_working(&mut stream).await?;

    Ok(())
}

// Listen UDP socket, until a connection comes with a valid port number,
// assume it's the TCP port of the receiver.
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
            message::send_msg(Message::Error(format!("Connection refused: {}", String::from_utf8(detail)?)));
            // Add this socket to black list if connection being refused to avoid future attemp.
            BLACK_LIST.lock().unwrap().push(*socket);
        },
        Operation::ConnError => { 
            let detail = utils::recv_content(&mut stream, response.length as usize).await?;
            //println!("Error when connecting: {}", String::from_utf8(detail)?);
            message::send_msg(Message::Error(format!("in connection: {}", String::from_utf8(detail)?)))
        },
        _ =>  println!("Unknow instruction in reponse"),
    }

    Ok(None)
}

async fn timer(expire: u8, rx: mpsc::Receiver<bool>) {
    let start = Instant::now();

    while start.elapsed().as_secs() < (expire * 60) as u64 {
        let t = (expire * 60) as u64 - start.elapsed().as_secs();
        message::send_msg(Message::Time(t));
        
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;
        if Ok(true) == rx.try_recv() {
            return;
        }
    }

    message::send_msg(Message::Fatal(format!("no connection in time")));
}

// After the connection established, start sending files and messages from here.
async fn start_working(stream: &mut TcpStream) -> Result<()> {
    let mut id = 0;

    if request_disconnect(stream, &mut id).await? {
        message::send_msg(Message::Status(format!("Ready to shutdown")));
    }

    // shutdown service.
    //stream.shutdown(std::net::Shutdown::Both)?;
    message::send_msg(Message::Done);
    Ok(())
}

async fn request_disconnect(stream: &mut TcpStream, id: &mut u16) -> Result<bool> {
    *id = utils::inc_one_u16(*id);
    utils::send_ins(stream, *id, Operation::EndConn, None).await?;
    let reply = utils::recv_ins(stream).await?;

    match (reply.id, reply.operation) {
        (id, Operation::RequestSuccess) => return Ok(true),
        (_, _) => message::send_msg(Message::Error(format!("wrong response for disconnection"))),
    }

    Ok(false)
}
