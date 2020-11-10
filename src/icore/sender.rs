use anyhow::{anyhow, Result};
use async_std::net::{UdpSocket, TcpStream};
use log::{debug, info};
use std::path::PathBuf;
use std::net::SocketAddr;
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};
use super::arg::SendArg;
use super::currentfile::CurrentFile;
use super::instruction::Operation;
use super::message::{Message, self};
use super::utils;

// Store refused sockets into a black list.
lazy_static::lazy_static! {
    static ref BLACK_LIST: Mutex<Vec<SocketAddr>> = Mutex::new(Vec::new());
    static ref ID: Mutex<u16> = Mutex::new(1);
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
    start_working(&mut stream, arg).await?;

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

    match validate_reply(&mut stream, 0).await {
        Ok((true, _)) => Ok(Some(stream)),
        Ok((false, detail)) => {
            message::send_msg(Message::Error(format!("Connection refused: {}", detail)));
            BLACK_LIST.lock().unwrap().push(*socket);
            Ok(None)
        },
        Err(e) => {
            message::send_msg(Message::Error(format!("Error trying connecting TCP: {}", e)));
            Ok(None)
        }
    }
}

// Wait for `expire` minutes before terminate the process.
// Can be interrupted by the signal from parent function.
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
async fn start_working(stream: &mut TcpStream, arg: SendArg) -> Result<()> {
    if arg.files.is_some() {
        send_files(stream, &arg.files.unwrap()).await?;
    }

    if request_disconnect(stream).await? {
        message::send_msg(Message::Status(format!("Ready to shutdown")));
    }

    message::send_msg(Message::Done);
    Ok(())
}

async fn send_files(stream: &mut TcpStream, files: &Vec<PathBuf>) -> Result<()> {
    for file in files {
        if let Err(e) = send_single_file(stream, file).await {
            message::send_msg(Message::Error(format!("Error sending file {:?} : {}", file, e)));
        }

        incre_id();
    }

    Ok(())
}

// If any error happens or receiver chooses skip, skip this file.
async fn send_single_file(stream: &mut TcpStream, file: &PathBuf) -> Result<()> {
    let current_file = CurrentFile::from(file)?;
    if !send_file_meta(stream, &current_file).await? {
        return Ok(());
    }
    debug!("Send file done");

    Ok(())
}

// Send file name and size as metainfo to receiver.
async fn send_file_meta(stream: &mut TcpStream, file: &CurrentFile) -> Result<bool> {
    let meta = file.meta_to_string();
    let id = read_id();
    utils::send_ins(stream, id, Operation::StartSendFile, Some(&meta)).await?;

    match validate_reply(stream, id).await? {
        (true, _) => Ok(true),
        (false, detail) => {
            message::send_msg(Message::Status(detail));
            Ok(false)
        }
    }
}

async fn request_disconnect(stream: &mut TcpStream) -> Result<bool> {
    let id = read_id();
    utils::send_ins(stream, id, Operation::Disconnect, None).await?;

    match validate_reply(stream, id).await? {
        (true, _) => { Ok(true) },
        (false, detail) => {
            message::send_msg(Message::Fatal(format!("disconnection request refused: {}", detail)));
            Ok(false)
        }
    }
}

// Validate the reply id and the reply operation.
// For abnormal reply, read the details as well.
async fn validate_reply(stream: &mut TcpStream, id: u16) -> Result<(bool, String)> {
    let reply = utils::recv_ins(stream).await?;
    if reply.id != id {
        return Err(anyhow!("wrong id in reply"));
    }

    let detail = if reply.buffer {
        get_reply_content(stream, reply.length as usize).await?
    } else {
        String::new()
    };

    match reply.operation {
        Operation::RequestSuccess => Ok((true, detail)),
        Operation::RequestRefuse => Ok((false, detail)),
        Operation::RequestError => Err(anyhow!(detail)),
        _ => Err(anyhow!("Unknown reply")),
    }
}

// Helper function to read details for validate_reply().
async fn get_reply_content(stream: &mut TcpStream, length: usize) -> Result<String> {
    let detail = utils::recv_content(stream, length).await?;

    Ok(String::from_utf8(detail)?)
}

// Increment ID by 1. If it reaches the boundary of U16, set it to 1.
// 0 is reservered.
fn incre_id() {
    let mut id = ID.lock().unwrap();
    *id = if *id < u16::MAX { *id + 1 } else {1};
}

fn read_id() -> u16 {
    let id = ID.lock().unwrap();

    *id
}