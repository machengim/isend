use anyhow::{anyhow, Result};
use async_std::prelude::*;
use async_std::fs::OpenOptions;
use async_std::net::{TcpListener, TcpStream, UdpSocket};
use std::path::PathBuf;
use std::sync::mpsc;
use super::arg::{OverwriteStrategy, RecvArg};
use super::currentfile::CurrentFile;
use super::instruction::{Instruction, Operation};
use super::message::{self, Message};
use super::utils;

pub async fn launch(arg: RecvArg) -> Result<()> {
    log::info!("Start receiver function");
    let tcp_socket = TcpListener::bind(("0.0.0.0", 0)).await?;
    let tcp_port = tcp_socket.local_addr()?.port();
    log::debug!("Listen on TCP port {}", tcp_port);

    let (tx, rx) = mpsc::channel();
    let arg_code = arg.code.clone();
    async_std::task::spawn(async move {
        if let Err(e) = broadcast_udp(tcp_port, arg_code, rx).await {
            message::send_msg(Message::Fatal(format!("UDP broadcast issue: {}", e)));
        }
    });

    // After the connection established, stop the broadcast.
    let mut stream = listen_tcp_conn(&tcp_socket, arg.password.as_ref()).await?;
    tx.send(true)?;

    start_recving(&mut stream, arg).await?;

    Ok(())
}

// Send UDP broadcast 10 times unless signal received
// local_port: TCP port of local machine(receiver)
// target_port: UDP port of remote machine(sender) 
async fn broadcast_udp(local_port: u16, target_port: u16, rx: mpsc::Receiver<bool>) -> Result<bool> {
    let udp_socket = UdpSocket::bind("0.0.0.0:0").await?;
    udp_socket.set_broadcast(true)?;

    for _ in 0..10 {
        udp_socket.send_to(&u16::to_be_bytes(local_port), ("255.255.255.255", target_port)).await?;
        log::debug!("UDP broadcast sent to port {}", target_port);
        async_std::task::sleep(std::time::Duration::from_secs(5)).await;

        // Check message in channel
        if let Ok(true) = rx.try_recv() {
            return Ok(true);
        }
    }

    Err(anyhow!("No connection established"))
}

// Wait for tcp connection on the tcp socket and validate it.
// TODO: terminate after time runs out
async fn listen_tcp_conn(socket: &TcpListener, password: Option<&String>) -> Result<TcpStream> {

    loop {
        let (mut stream, addr) = socket.accept().await?;
        log::info!("Receive connection request from {}", &addr);

        let ins = utils::recv_ins(&mut stream).await?;
        match ins.operation {
            Operation::Connect => {
                match valiate_tcp_conn(&mut stream, &ins, password).await {
                    Ok(true) => {
                        utils::send_ins(&mut stream, 0, Operation::RequestSuccess, None).await?;
                        message::send_msg(Message::Status(format!("Connection established\n")));
                        return Ok(stream);
                    },
                    Ok(false) => {
                        let reply = format!("Invalid password");
                        utils::send_ins(&mut stream, 0, Operation::RequestRefuse, Some(&reply)).await?;
                        message::send_msg(Message::Status(format!("Connection refused: {}", reply)));
                    }
                    Err(e) => {
                        let reply = format!("Get error when validating tcp connection: {}", e);
                        utils::send_ins(&mut stream, 0, Operation::RequestError, Some(&reply)).await?;
                        message::send_msg(Message::Status(reply));
                    }
                }
            },
            _ => { log::debug!("Unknown operation code when expecting connection request")},
        }
    }
}

// validate tcp connection, if both passwords are provided, check the password content.
async fn valiate_tcp_conn(stream: &mut TcpStream, ins: &Instruction, password: Option<&String>)
    -> Result<bool> {

    match (ins.buffer, password.is_some()) {
        (false, false) => Ok(true),
        (true, true) => {
            if compare_pass(stream, ins, password.unwrap()).await? {
                Ok(true)
            } else {
                Ok(false)
            }},
        _ => Ok(false),
    }
}

async fn compare_pass(stream: &mut TcpStream, ins: &Instruction, password: &String)
    -> Result<bool> {
    
    let buf = utils::recv_content(stream, ins.length as usize).await?;
    let req_pass = String::from_utf8(buf)?;
    log::debug!("Local password: {}, remote password: {}", password, req_pass);

    Ok(&req_pass == password)
}

async fn start_recving(stream: &mut TcpStream, arg:RecvArg) -> Result<()> {
    let mut arg = arg;
    let mut current_file = CurrentFile::default();

    loop {
        let ins = utils::recv_ins(stream).await?;
        
        match ins.operation {
            Operation::StartSendFile => recv_file_meta(stream, &ins, &mut current_file, &arg).await?,
            Operation::SendFileContent => recv_file_content(stream, &ins, &mut current_file).await?,
            Operation::EndSendFile => recv_file_end(stream, &ins, &mut current_file).await?,
            Operation::StartSendDir => recv_dir(stream, &ins, &mut arg).await?,
            Operation::EndSendDir => recv_dir_end(stream, &ins, &mut arg).await?,
            Operation::SendMsg => recv_msg(stream, &ins).await?,
            Operation::Disconnect => shutdown(stream, ins.id).await?,
            _ => return Err(anyhow!("Unknown request instruction")),
        }
    }
}

async fn recv_dir(stream: &mut TcpStream, ins: &Instruction, arg: &mut RecvArg) -> Result<()> {
    let dir_name_buf = utils::recv_content(stream, ins.length as usize).await?;
    let dir_name = String::from_utf8(dir_name_buf)?;
    let (child_path, needs_create) = match get_valid_path(&dir_name, arg) {
        Some((path, need)) => (path, need),
        None => {
            reply_refuse(stream, ins.id, "Directory refused: user chose skip").await?;
            return Ok(());
        }
    };

    // Only if the overwrite strategy is `rename`, a new directory needs to be created.
    if needs_create && !create_dir(&child_path) {
        let detail = format!("Cannot create directory on receiver");
        reply_error(stream, ins.id, &detail).await?;

        return Ok(());
    }

    arg.dir = child_path;
    log::debug!("Current working dir is: {:?}", &arg.dir);
    reply_success(stream, ins.id).await?;

    Ok(())
}

fn create_dir(path: &PathBuf) -> bool {
    if let Err(e) = std::fs::create_dir(&path) {
        message::send_msg(Message::Error(format!("Create dir failed: {}", e)));
        return false;
    }

    true
}

async fn recv_dir_end(stream: &mut TcpStream, ins: &Instruction, arg:&mut RecvArg) -> Result<()> {
    let current = arg.dir.file_name().unwrap();
    message::send_msg(Message::Status(format!("Finish receiving directory: {:?}", current)));
    arg.dir.pop();
    log::debug!("Current working dir: {:?}", &arg.dir);
    reply_success(stream, ins.id).await?;

    Ok(())
}

// Read file meta info from sender and prepare the file descriptor.
// If file name already existed, perform according to the overwrite strategy.
// TODO: check available disk space.
async fn recv_file_meta(stream: &mut TcpStream, ins: &Instruction, 
    file: &mut CurrentFile, arg: &RecvArg) -> Result<()> {
    
    // If the previous file is still transmitting, refuse current file and print error message.
    // Return OK so the loop in parent function will continue.
    if file.fd.is_some() {
        reply_error(stream, ins.id, "Previous file not finished").await?;
        return Ok(());
    }

    let meta = utils::recv_content(stream, ins.length as usize).await?;
    let (size, name) = match CurrentFile::meta_from_string(&String::from_utf8(meta)?) {
        Ok((s, n)) => (s, n),
        Err(_) => {
            reply_error(stream, ins.id, "Cannot read file meta info").await?;
            return Ok(());
        },
    };

    log::debug!("File name: {}, size: {}", &name, size);
    match get_valid_path(&name, arg) {
        Some((path, _)) => {
            prepare_file(path, size, file).await?;
            reply_success(stream, ins.id).await?;
            log::debug!("Prepared file: {:?}", file);
        },
        None => {
            reply_refuse(stream, ins.id, "File refused: user chose skip").await?;
            return Ok(());
        }
    }

    Ok(())
}

async fn recv_file_content(stream: &mut TcpStream, ins: &Instruction, file: &mut CurrentFile) -> Result<()> {
    let content_buf = utils::recv_content(stream, ins.length as usize).await?;
    let mut fd = file.must_get_fd()?;

    fd.write_all(&content_buf).await?;
    file.transmitted += ins.length as u64;
    message::send_msg(Message::Progress(file.get_progress()));

    Ok(())
}

async fn recv_file_end(stream: &mut TcpStream, ins: &Instruction, file: &mut CurrentFile) -> Result<()> {
    if file.fd.is_none() {
        return Err(anyhow!("File not opened"));
    }

    // Reset the current file when receiving the end file command.
    *file = CurrentFile::default();
    message::send_msg(Message::FileEnd);
    utils::send_ins(stream, ins.id, Operation::RequestSuccess, None).await?;

    Ok(())
}

async fn recv_msg(stream: &mut TcpStream, ins: &Instruction) -> Result<()> {
    let msg_buf = utils::recv_content(stream, ins.length as usize).await?;
    let msg = String::from_utf8(msg_buf)?;
    message::send_msg(Message::Status(format!("\nMessage received: \"{}\"", &msg)));
    reply_success(stream, ins.id).await?;

    Ok(())
}

fn get_valid_path(name: &String, arg: &RecvArg) -> Option<(PathBuf, bool)> {
    let mut path = PathBuf::new();
    let mut overwrite = arg.overwrite.clone();
    let mut i = 0u16;
    let mut renamed = false;

    path.push(arg.dir.clone());
    path.push(name);

    let dir_name = path.file_name().unwrap();
    if path.is_dir() {
        message::send_msg(Message::Status(format!("Start receiving directory: {:?}", dir_name)));
    }

    let ftype = if path.is_file() { "file" } else { "directory" };
    let existed = path.is_file() || path.is_dir();

    if existed && overwrite == OverwriteStrategy::Ask {
        let info = format!("Alert: {} {:?} already existed", &ftype, &dir_name);
        message::send_msg(Message::Status(info));
    }

    while path.is_file() || path.is_dir() {
        match overwrite {
            OverwriteStrategy::Ask => {
                overwrite = OverwriteStrategy::ask();
            },
            OverwriteStrategy::Overwrite => {
                break;
            },
            OverwriteStrategy::Rename => {
                path.pop();
                path.push(format!("{}_{}", i, name));
                i += 1;     // Assume u16 is more than enough to try.
                renamed = true;
            },
            // Note here none is used for skip name with same name
            OverwriteStrategy::Skip => return None,
        }
    }

    if renamed {
        message::send_msg(Message::Status(format!("Renamed {} to {:?}", 
            &ftype, path.file_name().unwrap())));
    }

    // The path needs to be created if it didn't exist or
    // it existed but user chose to rename it.
    Some((path, !existed || renamed))
}

async fn prepare_file(path: PathBuf, size: u64, file: &mut CurrentFile) -> Result<()> {
    log::debug!("Creating file: {:?}", &path);
    let filename = String::from(path.file_name().unwrap().to_str().unwrap());
    let path_str = path.to_str().unwrap();
    let fd = OpenOptions::new().write(true).create(true).open(path_str).await?;

    file.path = path;
    file.name = filename;
    file.size = size;
    file.fd = Some(fd);

    Ok(())
}

async fn shutdown(stream: &mut TcpStream, id: u16) -> Result<()> {
    utils::send_ins(stream, id, Operation::RequestSuccess, None).await?;
    //stream.shutdown(std::net::Shutdown::Both)?;
    message::send_msg(Message::Done);

    Ok(())
}

async fn reply_success(stream: &mut TcpStream, id: u16) -> Result<()> {
    utils::send_ins(stream, id, Operation::RequestSuccess, None).await?;

    Ok(())
}

async fn reply_error(stream: &mut TcpStream, id: u16, detail_str: &str) -> Result<()> {
    let detail = String::from(detail_str);
    utils::send_ins(stream, id, Operation::RequestError, Some(&detail)).await?;
    message::send_msg(Message::Error(detail));

    Ok(())
}

async fn reply_refuse(stream: &mut TcpStream, id: u16, detail_str: &str) -> Result<()> {
    let detail = String::from(detail_str);
    utils::send_ins(stream, id, Operation::RequestRefuse, Some(&detail)).await?;
    message::send_msg(Message::Status(detail));

    Ok(())
}
