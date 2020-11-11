use anyhow::{anyhow, Result};
use async_std::fs::OpenOptions;
use async_std::prelude::*;
use async_std::net::{UdpSocket, TcpStream};
use async_std::task::block_on;
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
    message::send_msg(Message::Status(format!("Connection established\n")));

    // Start sending files and messages.
    start_sending(&mut stream, arg).await?;

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
            log::debug!("Found socket in black list {}", &socket);
            continue;
        }

        log::debug!("Connection request from {}", socket);
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
    log::debug!("Timer in {} minutes", expire);
    let start = Instant::now();

    while start.elapsed().as_secs() < (expire * 60) as u64 {
        // Note the type cast should come first to avoid `expire` overflow.
        let t = (expire as u64 * 60) - start.elapsed().as_secs();
        message::send_msg(Message::Time(t));
        
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;
        if Ok(true) == rx.try_recv() {
            return;
        }
    }

    message::send_msg(Message::Fatal(format!("no connection in time")));
}

// After the connection established, start sending files and messages from here.
async fn start_sending(stream: &mut TcpStream, arg: SendArg) -> Result<()> {
    if arg.files.is_some() {
        send_files(stream, &arg.files.unwrap())?;
    }

    if arg.msg.is_some(){
        send_message(stream, &arg.msg.unwrap()).await?;
    }

    if request_disconnect(stream).await? {
        log::debug!("Ready to shutdown");
    }

    message::send_msg(Message::Done);
    Ok(())
}

// identify files and dirs and process them accordingly.
// Remove `async` of this function to avoid async recursion.
fn send_files(stream: &mut TcpStream, files: &Vec<PathBuf>) -> Result<()> {
    for file in files {
        if file.is_file() {
            if let Err(e) = block_on(send_single_file(stream, file)) {
                message::send_msg(Message::Error(format!("Error sending file {:?} : {}", file, e)));
            }
        } else if file.is_dir() {
            if let Err(e) = block_on(send_dir(stream, file)) {
                message::send_msg(Message::Error(format!("Error sending dir {:?} : {}", file, e)));
            }
        } else {
            return Err(anyhow!("Unknow file type"));
        }
    }

    Ok(())
}

// 1. Check the directory name first. TODO: if overwrite strategy is overwrite, do not create the dir.
// 2. Collect all paths inside the current dir and pass them to send_files() function.
// Async function doesn't support recursion.
async fn send_dir(stream: &mut TcpStream, dir: &PathBuf) -> Result<()> {
    let id = read_id();
    let dir_name = dir.file_name().unwrap().to_str().unwrap().to_string();

    message::send_msg(Message::Status(format!("Start sending directory: \"{}\"", &dir_name)));
    utils::send_ins(stream, id, Operation::StartSendDir, Some(&dir_name)).await?;
    incre_id();

    // If request being refused, abort the following action.
    // Message has been sent to UI through the process_reply() function.
    if !process_reply(stream, id).await? {
        return Ok(());
    }

    let mut paths = Vec::new();
    for entry in dir.read_dir()? {
        if let Ok(entry) = entry {
            paths.push(entry.path());
        }
    }

    // Problem unsolved: once the recursion starts, the return type is required to be `dyn Future`.
    send_files(stream, &paths)?;
    
    send_dir_end(stream).await?;
    message::send_msg(Message::Status(format!("Finish sending directory: \"{}\"", &dir_name)));

    Ok(())
}

async fn send_dir_end(stream: &mut TcpStream) -> Result<()> {
    let id = read_id();
    utils::send_ins(stream, id, Operation::EndSendDir, None).await?;
    incre_id();

    process_reply(stream, id).await?;

    Ok(())
}

// If any error happens or receiver chooses skip, skip this file.
async fn send_single_file(stream: &mut TcpStream, file: &PathBuf) -> Result<()> {
    let mut current_file = CurrentFile::from(file)?;
    if !send_file_meta(stream, &current_file).await? {
        return Ok(());
    }

    send_file_content(stream, &mut current_file).await?;

    send_file_end(stream).await?;

    Ok(())
}

// Send file name and size as metainfo to receiver.
async fn send_file_meta(stream: &mut TcpStream, file: &CurrentFile) -> Result<bool> {
    let meta = file.meta_to_string();
    let id = read_id();
    utils::send_ins(stream, id, Operation::StartSendFile, Some(&meta)).await?;
    incre_id();

    process_reply(stream, id).await
}

async fn send_file_content(stream: &mut TcpStream, f: &mut CurrentFile) -> Result<()> {
    log::debug!("Sending file content");
    let mut file = OpenOptions::new().read(true).open(f.path.clone()).await?;
    let chunk_size = 0x800000;  // 8M size

    loop {
        let mut chunk = Vec::with_capacity(chunk_size);
        let length = file.by_ref().take(chunk_size as u64).read_to_end(&mut chunk).await?;
        if length == 0 { break; }

        utils::send_ins_bytes(stream, read_id(), Operation::SendFileContent, &chunk).await?;
        f.transmitted += length as u64;
        message::send_msg(Message::Progress(f.get_progress()));
    }

    message::send_msg(Message::FileEnd);
    incre_id();

    Ok(())
}

async fn send_file_end(stream: &mut TcpStream) -> Result<bool> {
    let id = read_id();
    utils::send_ins(stream, id, Operation::EndSendFile, None).await?;

    incre_id();
    process_reply(stream, id).await
}

async fn send_message(stream: &mut TcpStream, msg: &String) -> Result<bool> {
    let id = read_id();
    utils::send_ins(stream, id, Operation::SendMsg, Some(msg)).await?;

    incre_id();
    process_reply(stream, id).await
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

async fn process_reply(stream: &mut TcpStream, id: u16) -> Result<bool> {
    match validate_reply(stream, id).await? {
        (true, _) => Ok(true),
        (false, detail) => {
            message::send_msg(Message::Status(detail));
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