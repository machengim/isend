use anyhow::{anyhow, Result};
use async_std::net::{TcpListener, TcpStream, UdpSocket};
use async_std::prelude::*;
use async_std::task::block_on;
use async_std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc;
use crate::{arg, utils};
use crate::message::Message;
use crate::instruction::{Instruction, Operation};

pub struct Sender {
    id: u16,
    files: Option<Vec<PathBuf>>,
    msg: Option<String>,
    password: Option<String>,
    stream: TcpStream,
    tx: mpsc::Sender<Message>,
}

pub struct Receiver {
    dir: PathBuf,
    overwrite: arg::OverwriteStrategy,
    stream: TcpStream,
    tx: mpsc::Sender<Message>,
}

impl Sender {
    pub async fn launch(arg: arg::SendArg, tx: mpsc::Sender<Message>) -> Result<()> {
        let udp = UdpSocket::bind(("0.0.0.0", arg.port)).await?;
        let port = udp.local_addr()?.port();
        let pass = utils::rand_range(0, 255);
    
        let code = generate_code(port, pass);
        println!("Connection code: {}", code);
    
        let dest = listen_upd(&udp, pass).await?;
        let stream = TcpStream::connect(dest).await?;
    
        let mut sender = Sender::new(&arg, stream, tx);
        sender.connect().await?;
        sender.send_files().await?;
        sender.send_msg().await?;
        sender.disconnect().await?;
    
        Ok(())
    }

    fn new(arg: &arg::SendArg, stream: TcpStream, tx: mpsc::Sender<Message>) -> Sender{
        Sender {
            id: 0,
            files: arg.files.clone(),
            msg: arg.msg.clone(),
            password: arg.password.clone(),
            stream,
            tx,
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
        if let Ok(2) = self.send_file_name(f).await {
            return Ok(());
        }
        //println!("Sending file: {:?}", f);
        let status = Message::Status(format!("Sending file: {:?}", f));
        self.tx.send(status)?;
        self.send_file_meta(f).await?;
        self.send_file_content(f).await?;

        Ok(())
    }

    async fn send_file_name(&mut self, f: &PathBuf) -> Result<u8> {
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
            Operation::RequestSuccess => Ok(1),
            Operation::AbortFile => Ok(2),
            _ => Err(anyhow!("Wrong response got when sending file name")),
        }
    }

    async fn send_file_meta(&mut self, f: &PathBuf) -> Result<()> {
        let ins = Instruction {
            id: self.id,
            operation: Operation::SendMeta,
            buffer: true,
            length: 8,
        };

        let file_size = std::fs::metadata(f)?.len();
        send(&mut self.stream, &ins, Some(Box::new(&file_size.to_be_bytes()))).await?;

        Ok(())
    }

    async fn send_file_content(&mut self, f: &PathBuf) -> Result<()> {
        let mut file = OpenOptions::new().read(true).open(f).await?;
        let chunk_size = 0x8000;    // 32k size
        let file_size = std::fs::metadata(f)?.len();
        let mut sent_size = 0u64;
        let mut progress = 0u8;
        
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

            sent_size += length as u64;
            let current_progress = (sent_size * 100 / file_size) as u8;
            if current_progress > progress {
                progress = current_progress;
                self.tx.send(Message::Progress(progress))?;
            }
        }

        let ins = Instruction{id: self.id, operation: Operation::EndContent, buffer: false, length: 0};
        send(&mut self.stream, &ins, None).await?;
        self.id += 1;

        Ok(())
    }

    async fn send_msg(&mut self) -> Result<()> {
        if let Some(msg) = self.msg.as_ref() {
            let request = Instruction {
                id: self.id,
                operation: Operation::PreSendMsg,
                buffer: true,
                length: msg.len() as u16,
            };

            send(&mut self.stream, &request, Some(Box::new(msg.as_bytes()))).await?;
            self.id += 1;
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        let ins = Instruction {
            id: self.id,
            operation: Operation::EndConn,
            buffer: false,
            length: 0,
        };

        send(&mut self.stream, &ins, None).await?;
        self.id += 1;
        self.tx.send(Message::Done)?;

        Ok(())
    }
}

impl Receiver {
    pub async fn launch(arg: arg::RecvArg, txu: mpsc::Sender<Message>) -> Result<()>{
        let (udp_port, udp_pass) = parse_code(&arg.code)?;

        let tcp_socket = TcpListener::bind(("0.0.0.0", arg.port)).await?;
        let tcp_port = tcp_socket.local_addr()?.port();

        let code = generate_code(tcp_port, udp_pass);
        let (tx, rx) = mpsc::channel::<bool>();
        let txu2 = txu.clone();
        std::thread::spawn(move || {
            if !block_on(udp_broadcast(code, udp_port, rx)).unwrap() {
                if let Err(e) = txu2.send(Message::Error(format!("Cannot connect to sender in time"))) {
                    println!("Cannot send error to main thread: {}", e);
                    std::process::exit(1);
                }
            } 
        });
        let stream = listen_tcp(&tcp_socket, arg.password.as_ref()).await?;
        let mut receiver = Receiver::new(&arg, stream, txu);
        tx.send(true)?;

        let ins = Instruction{id: 0, operation: Operation::ConnSuccess, buffer: false, length: 0};
        send(&mut receiver.stream, &ins, None).await?;

        receiver.process().await?;

        Ok(())
    }

    fn new(arg: &arg::RecvArg, stream: TcpStream, tx: mpsc::Sender<Message>) -> Receiver {
        Receiver {
            dir: arg.dir.clone(),
            overwrite: arg.overwrite.clone(),
            stream,
            tx,
        }
    }

    async fn process(&mut self) -> Result<()> {
        loop {
            let ins = recv_ins(&mut self.stream).await?;

            match ins.operation{
                Operation::PreSendFile => self.process_file(&ins).await?,
                Operation::PreSendMsg => self.process_msg(&ins).await?,
                Operation::EndConn => break,
                // TODO: handle other operations
                _ => break,
            }
        }
        self.tx.send(Message::Done)?;

        Ok(())
    }

    // Check the file path existed in the receiver's machine
    // and change file name if necessary.
    fn get_valid_path(&self, orignal_path: &str) -> Result<PathBuf> {
        let mut path = PathBuf::new();
        let mut overwrite = self.overwrite.clone();
        path.push(&self.dir);
        path.push(orignal_path);
        
        while path.is_file() || path.is_dir() {
            match overwrite {
                arg::OverwriteStrategy::Ask => {
                    overwrite = arg::OverwriteStrategy::ask();
                },
                arg::OverwriteStrategy::Overwrite => break,
                arg::OverwriteStrategy::Rename => path = self.get_valid_path_seq(orignal_path),
                // Note here none is used for skip name with same name
                arg::OverwriteStrategy::Skip => return Err(anyhow!("Command to skip current file")),
            }
        }

        Ok(path)
    }

    fn get_valid_path_seq(&self, orignal_path: &str) -> PathBuf {
        let mut path = PathBuf::new();
        path.push(&self.dir);
        let mut i = 0;
        path.push(format!("{}_{}", i, orignal_path));

        while path.is_dir() || path.is_file() {
            i += 1;
            path.pop();
            path.push(format!("{}{}", orignal_path, i));
        }
    
        self.tx.send(Message::Status(format!("File renamed as {:?}", &path)))
            .unwrap_or(println!("Cannot send message to main thread"));

        path
    }

    // Work flow: receive instruction to create file -> Read file name ->
    // Create file -> Send success reply -> receive instruction to send file content
    // Read file content -> Receive instruction to end -> Reply success.
    async fn process_file(&mut self, ins: &Instruction) -> Result<()> {
        let mut file = match self.process_file_name(ins).await {
            Ok(f) => f,
            Err(e) => {
                println!("{}", e);
                let reply = Instruction{
                    id: ins.id, 
                    operation: Operation::AbortFile,
                    buffer: false,
                    length: 0,
                };
                send(&mut self.stream, &reply, None).await?;
                return Ok(())
            }
        };

        let mut ins: Instruction;
        let mut file_size = 0u64;
        let mut finished_size = 0u64;
        let mut progress = 0u8;

        loop {
            ins = recv_ins(&mut self.stream).await?;
            match ins.operation {
                Operation::SendMeta => file_size = self.process_file_size(&ins).await?,
                Operation::SendContent => self.process_file_content(&ins, &mut file, &mut finished_size).await?,
                Operation::EndContent => break,
                _ => return Err(anyhow!("Unknown instruction in file sending")),
            }

            if file_size > 0 && finished_size > 0 {
                let current_progress: u8 = if finished_size * 100 / file_size > 100 {100} else {(finished_size * 100 / file_size) as u8};

                if current_progress as u8 > progress {
                    progress = current_progress;
                    self.tx.send(Message::Progress(progress))?;
                }
            }
        }

        ins = Instruction{id: ins.id, operation: Operation::RequestSuccess, buffer: false, length: 0};
        send(&mut self.stream, &ins, None).await?;

        Ok(())
    }

    async fn process_file_name(&mut self, ins: &Instruction) -> Result<File> {
        let filename_buf = recv_content(&mut self.stream, ins.length as usize).await?;
        let dest_file = std::str::from_utf8(&filename_buf)?;
        println!("Receiving file: {}", &dest_file);
        let filename = self.get_valid_path(dest_file)?;
        let file = OpenOptions::new().write(true).create(true).open(filename).await?;
        let ins = Instruction {
            id: ins.id,
            operation: Operation::RequestSuccess,
            buffer: false,
            length: 0,
        };
        send(&mut self.stream, &ins, None).await?;

        Ok(file)
    }

    async fn process_file_size(&mut self, ins: &Instruction) -> Result<u64> {
        let buf = recv_content(&mut self.stream, ins.length as usize).await?;
        let mut buf_arr = [0u8; 8];
        buf_arr.clone_from_slice(&buf[..]);
        let file_size = u64::from_be_bytes(buf_arr);

        Ok(file_size)
    }

    async fn process_file_content(&mut self, ins: &Instruction, file: &mut File, finished: &mut u64) 
        -> Result<()> {

        let length = ins.length;
        let content_buf = recv_content(&mut self.stream, length as usize).await?;
        file.write_all(&content_buf).await?;
        *finished += length as u64;

        Ok(())
    }

    async fn process_msg(&mut self, ins: &Instruction) -> Result<()> {
        let length = ins.length;
        let content_buf = recv_content(&mut self.stream, length as usize).await?;
        println!("Get message: {}", String::from_utf8(content_buf)?);

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
            } else {
                let ins = Instruction {id: 0, operation: Operation::ConnRefuse, buffer: false, length: 0};
                send(&mut stream, &ins, None).await?;
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

    let buf = ins.encode();
    stream.write_all(&buf).await?;

    if let Some(s) = content {
        stream.write_all(&s).await?;
    }

    Ok(())
}

async fn recv_ins(stream: &mut TcpStream) -> Result<Instruction> {
    let mut buf = Vec::with_capacity(6);
    stream.by_ref().take(6).read_to_end(&mut buf).await?;
    let ins = Instruction::decode(&buf)?;

    Ok(ins)
}

// Use take().read_to_end() instead of read() as the latter causes reading problem.
async fn recv_content(stream: &mut TcpStream, length: usize) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(length);
    stream.by_ref().take(length as u64).read_to_end(&mut buf).await?;

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