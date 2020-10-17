use anyhow::Result;
use async_std::net::{TcpListener, TcpStream, UdpSocket};
use async_std::prelude::*;
use async_std::task::block_on;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc;
use crate::{arg, instruction::Instruction, utils, notify};
use crate::communication::Message;

pub struct Sender {
    id: u16,
    files: Option<Vec<PathBuf>>,
    msg: Option<String>,
    password: Option<String>,
    stream: TcpStream,
}

pub struct Receiver {
    dir: PathBuf,
    overwrite: arg::OverwriteStrategy,
    stream: TcpStream,
}

impl Sender {
    pub async fn launch(arg: arg::SendArg) -> Result<()> {
        let udp = UdpSocket::bind(("0.0.0.0", arg.port)).await?;
        let port = udp.local_addr()?.port();
        let pass = utils::rand_range(0, 255);
    
        let code = generate_code(port, pass);
        notify(Message::Status(code));
    
        let dest = listen_upd(&udp, pass).await?;
        let stream = TcpStream::connect(dest).await?;
    
        let mut sender = Sender::new(&arg, stream);
        sender.connect().await?;
    
        Ok(())
    }

    fn new(arg: &arg::SendArg, stream: TcpStream) -> Sender{
        Sender {
            id: 0,
            files: arg.files.clone(),
            msg: arg.msg.clone(),
            password: arg.password.clone(),
            stream,
        }
    }

    async fn connect(&mut self) -> Result<()> {
        let mut ins = Instruction::conn_without_pass(self.id);

        match &self.password {
            Some(pw) => {
                let pass = pw.clone();
                send(&mut self.stream, &mut ins, Some(Box::new(pass.as_bytes()))).await?;
            },
            None => send(&mut self.stream, &mut ins, None).await?,
        }

        Ok(())
    }
}

impl Receiver {
    pub async fn launch(arg: arg::RecvArg) -> Result<()>{
        let (udp_port, udp_pass) = parse_code(&arg.code)?;

        let tcp_socket = TcpListener::bind(("0.0.0.0", arg.port)).await?;
        let tcp_port = tcp_socket.local_addr()?.port();

        let code = generate_code(tcp_port, udp_pass);
        let (tx, rx) = mpsc::channel::<bool>();
        std::thread::spawn(move || {
            if !block_on(udp_broadcast(code, udp_port, rx)).unwrap() {
                notify(Message::Error("Cannot connect to sender in time".to_string()));
            } 
        });
        let stream = listen_tcp(&tcp_socket, arg.password.as_ref()).await?;
        let receiver = Receiver::new(&arg, stream);
        tx.send(true)?;

        Ok(())
    }

    fn new(arg: &arg::RecvArg, stream: TcpStream) -> Receiver {
        Receiver {
            dir: arg.dir.clone().unwrap(),
            overwrite: arg.overwrite.clone(),
            stream,
        }
    }
}

async fn listen_tcp(socket: &TcpListener, pass: Option<&String>) -> Result<TcpStream> {
    loop {
        let (mut stream, addr) = socket.accept().await?;
        println!("Get connection request from {}", addr);

        let mut buf = [0u8; 6];
        stream.read(&mut buf).await?;

        if let Ok(ins) = Instruction::decode(&buf) {
            if ins.buffer && ins.length > 0 && pass.is_some() {
                let mut buf = vec![0u8; ins.length as usize];
                stream.read(&mut buf).await?;
                if pass.unwrap() == &String::from_utf8(buf)? {
                    return Ok(stream);
                }
            } else if pass.is_none() {
                return Ok(stream);
            }
        }
    }
}

async fn listen_upd(socket: &UdpSocket, pass: u8) 
    -> Result<SocketAddr> {
    let mut buf = [0; 6];
    loop {
        let (_, addr) = socket.recv_from(&mut buf).await?;
        let code = std::str::from_utf8(&buf)?;
        let (dest_port, dest_pass) = parse_code(&code)?;
        
        if dest_pass == pass {
            return Ok(SocketAddr::new(addr.ip(), dest_port));
        }
    }
}

fn generate_code(port: u16, pass: u8) -> String {
    format!("{}{}", utils::dec_to_hex(port, 4), utils::dec_to_hex(pass as u16, 2))
}

// Convert a 6 char string to a tupe of port and password.
fn parse_code(code: &str) -> Result<(u16, u8)> {
    if code.len() != 6 || !utils::validate_hex_str(code) {
        let err = Error::new(ErrorKind::Other, "Cannot parse code");
        return Err(anyhow::Error::new(err));
    }

    let port = utils::hex_to_dec(&code[..4]);
    let pass = utils::hex_to_dec(&code[4..]) as u8;

    Ok((port, pass))
}

async fn send(stream: &mut TcpStream, ins: &mut Instruction,
    content: Option<Box<&[u8]>>) -> Result<()> {
    match &content {
        Some(bytes) => {
            ins.buffer = true;
            ins.length = bytes.len() as u16;
        },
        None => ins.buffer = false,
    }

    let buf = ins.encode();
    stream.write_all(&buf).await?;

    if let Some(s) = content {
        stream.write_all(&s).await?;
    }

    Ok(())
}

async fn udp_broadcast(code: String, udp_port: u16, rx: mpsc::Receiver<bool>) -> Result<bool> {
    let udp_socket = UdpSocket::bind("0.0.0.0:0").await?;
    udp_socket.set_broadcast(true)?;
    
    // TODO: this should be moved to a new thread.
    for _ in 0..10 {
        udp_socket.send_to(code.as_bytes(), ("255.255.255.255", udp_port)).await?;
        async_std::task::sleep(std::time::Duration::from_secs(5)).await;
        if let Ok(true) = rx.try_recv() {
            return Ok(true);
        }
    }

    Ok(false)
}