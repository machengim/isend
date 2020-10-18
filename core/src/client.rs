use anyhow::{anyhow, Result};
use async_std::net::{TcpListener, TcpStream, UdpSocket};
use async_std::prelude::*;
use async_std::task::block_on;
use async_std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc;
use crate::{arg, utils, notify};
use crate::communication::Message;
use crate::instruction::{Instruction, Operation};

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
        sender.send_files().await?;
    
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
        match &self.password {
            Some(pw) => {
                let ins = Instruction{
                    id: self.id, 
                    operation: Operation::ConnWithPass,
                    buffer: true,
                    length: pw.len() as u16,
                };
                let pass = pw.clone();
                send(&mut self.stream, &ins, Some(Box::new(pass.as_bytes()))).await?;
            },
            None => { 
                let ins = Instruction {
                    id: self.id,
                    operation: Operation::ConnWithoutPass,
                    buffer: false,
                    length: 0,
                };
                send(&mut self.stream, &ins, None).await?;
            }
        }

        self.id += 1;
        let ins = recv_ins(&mut self.stream).await?;

        match ins.operation {
            Operation::ConnSuccess => Ok(()),
            _ => Err(anyhow!("Connection failed")),
        }
    }

    async fn send_files(&mut self) -> Result<()> {
        let files = self.files.clone();
        if let Some(fs) = files {
            for f in fs {
                self.send_file_one(&f).await?;
            }
        }

        Ok(())
    }

    async fn send_file_one(&mut self, f: &PathBuf) -> Result<()> {
        self.send_file_name(f).await?;
        self.send_file_content(f).await?;

        Ok(())
    }

    async fn send_file_name(&mut self, f: &PathBuf) -> Result<()> {
        let filename = f.file_name().unwrap().to_str().unwrap();

        // Start sending file name.
        let request = Instruction {
            id: self.id,
            operation: Operation::PreSendFile,
            buffer: true,
            length: filename.len() as u16,
        };

        send(&mut self.stream, &request, Some(Box::new(filename.as_bytes()))).await?;
        self.id += 1;
        // Wait for response from receiver.
        let response = recv_ins(&mut self.stream).await?;

        match response.operation {
            Operation::RequestSuccess => Ok(()),
            _ => Err(anyhow!("Wrong response got when sending file name")),
        }
    }

    async fn send_file_content(&mut self, f: &PathBuf) -> Result<()> {
        let mut file = OpenOptions::new().read(true).open(f).await?;
        let chunk_size = 0x8000;    // 32k size
        
        loop {
            let mut chunk = Vec::with_capacity(chunk_size);
            let length = file.by_ref().take(chunk_size as u64).read_to_end(&mut chunk).await?;
            if length == 0 { break; }

            let ins = Instruction{
                id: self.id, 
                operation: Operation::SendContent, 
                buffer: true, 
                length: length as u16
            };

            send(&mut self.stream, &ins, Some(Box::new(&chunk))).await?;
            self.id += 1;
        }

        let ins = Instruction{id: self.id, operation: Operation::EndContent, buffer: false, length: 0};
        send(&mut self.stream, &ins, None).await?;
        self.id += 1;

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
        let mut receiver = Receiver::new(&arg, stream);
        tx.send(true)?;

        let ins = Instruction{id: 0, operation: Operation::ConnSuccess, buffer: false, length: 0};
        send(&mut receiver.stream, &ins, None).await?;

        receiver.process().await?;

        Ok(())
    }

    fn new(arg: &arg::RecvArg, stream: TcpStream) -> Receiver {
        Receiver {
            dir: arg.dir.clone(),
            overwrite: arg.overwrite.clone(),
            stream,
        }
    }

    async fn process(&mut self) -> Result<()> {
        loop {
            let ins = recv_ins(&mut self.stream).await?;

            match ins.operation{
                Operation::PreSendFile => self.process_file(&ins).await?,
                Operation::EndConn => break,
                // TODO: handle other operations
                _ => break,
            }
        }

        Ok(())
    }

    // Work flow: receive instruction to create file -> Read file name ->
    // Create file -> Send success reply -> receive instruction to send file content
    // Read file content -> Receive instruction to end -> Reply success.
    async fn process_file(&mut self, ins: &Instruction) -> Result<()> {
        let filename_buf = recv_content(&mut self.stream, ins.length as usize).await?;
        let filename = std::str::from_utf8(&filename_buf)?;
        let mut file = OpenOptions::new().append(true).create(true).open(filename).await?;
        let ins = Instruction {
            id: ins.id,
            operation: Operation::RequestSuccess,
            buffer: false,
            length: 0,
        };
        send(&mut self.stream, &ins, None).await?;

        let mut ins: Instruction;

        loop {
            ins = recv_ins(&mut self.stream).await?;
            match ins.operation {
                Operation::SendContent => self.process_file_content(&ins, &mut file).await?,
                Operation::EndContent => break,
                _ => return Err(anyhow!("Error when reading file content")),
            }
        }

        let ins = Instruction{id: ins.id, operation: Operation::RequestSuccess, buffer: false, length: 0};
        send(&mut self.stream, &ins, None).await?;

        Ok(())
    }

    async fn process_file_content(&mut self, ins: &Instruction, file: &mut File)
        -> Result<()> {

        let length = ins.length;
        let content_buf = recv_content(&mut self.stream, length as usize).await?;
        file.write_all(&content_buf).await?;

        Ok(())
    }
}

async fn listen_tcp(socket: &TcpListener, pass: Option<&String>) -> Result<TcpStream> {
    loop {
        let (mut stream, addr) = socket.accept().await?;
        println!("Get connection request from {}", addr);

        let ins = recv_ins(&mut stream).await?;

        if ins.buffer && ins.length > 0 && pass.is_some() {
            let buf = recv_content(&mut stream, ins.length as usize).await?;

            if pass.unwrap() == &String::from_utf8(buf)? {
                return Ok(stream);
            }
        } else if pass.is_none() {
            return Ok(stream);
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

async fn send(stream: &mut TcpStream, ins: &Instruction,
    content: Option<Box<&[u8]>>) -> Result<()> {

    println!("Sending {:?}", ins);
    let buf = ins.encode();
    stream.write_all(&buf).await?;

    if let Some(s) = content {
        stream.write_all(&s).await?;
    }

    Ok(())
}

async fn recv_ins(stream: &mut TcpStream) -> Result<Instruction> {
    let mut buf = [0u8; 6];
    stream.read(&mut buf).await?;
    let ins = Instruction::decode(&buf)?;
    println!("Get instruction {:?}", &ins);

    Ok(ins)
}

async fn recv_content(stream: &mut TcpStream, length: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; length];
    stream.read(&mut buf).await?;

    Ok(buf)
}

async fn udp_broadcast(code: String, udp_port: u16, rx: mpsc::Receiver<bool>) -> Result<bool> {
    let udp_socket = UdpSocket::bind("0.0.0.0:0").await?;
    udp_socket.set_broadcast(true)?;
    
    for _ in 0..10 {
        udp_socket.send_to(code.as_bytes(), ("255.255.255.255", udp_port)).await?;
        async_std::task::sleep(std::time::Duration::from_secs(5)).await;
        if let Ok(true) = rx.try_recv() {
            return Ok(true);
        }
    }

    Ok(false)
}