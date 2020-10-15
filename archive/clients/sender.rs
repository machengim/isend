use anyhow::Result;
use async_std::net::{UdpSocket, TcpStream};
use async_std::prelude::*;
use std::path::PathBuf;
use std::process::exit;
use std::net::SocketAddr;
use std::sync::mpsc;
use crate::{arguments, ui, utils};
use crate::protocol::{Instruction, Operation};

pub struct Sender {
    pub files: Option<Vec<PathBuf>>,
    pub msg: Option<String>,
    pub password: Option<String>,
    pub stream: TcpStream,
}

impl Sender {
    pub fn new(arg: arguments::SendArg, stream: TcpStream) -> Self {
        Sender {
            files: arg.files,
            msg: arg.msg,
            password: arg.password,
            stream,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        let ins = Instruction::init(&self.password);
        let buf = ins.encode();
        self.stream.write_all(&buf).await?;
        if let Some(password) = self.password.as_ref() {
            let buf = password.as_bytes();
            self.stream.write_all(buf).await?;
        }

        let mut buf = [0u8; 6];
        self.stream.read(&mut buf).await?;
        println!("Get response from receiver: {:?}", &buf);

        match Instruction::decode(&buf).operation {
            Operation::ConnSuccess => println!("Connection established"),
            Operation::ConnRefuse => { eprintln!("Connection being refused"); exit(1); }
            _ => { eprintln!("Unknow instruction received"); exit(1); }
        }

        Ok(())
    }
}

pub async fn launch(arg: arguments::SendArg) -> Result<()>{
    let socket = UdpSocket::bind(("0.0.0.0", arg.port)).await?;
    let pass = display_code(&socket)?;

    let (tx, rx) = mpsc::channel::<bool>();
    ui::timer::start_timer((&arg.expire * 60) as u64, rx);

    let dest = listen_upd(&socket, pass).await?;
    tx.send(true)?;

    let stream = TcpStream::connect(dest).await?;
    let mut sender = Sender::new(arg, stream);
    sender.connect().await?;

    Ok(())
}

async fn listen_upd(socket: &UdpSocket, pass: u16) 
    -> Result<SocketAddr> {

    let mut buf = [0; 6];
    loop {
        let (_, addr) = socket.recv_from(&mut buf).await?;
        let (dest_port, dest_pass) = utils::decode(std::str::from_utf8(&buf)?)
            .expect("Cannot parse code.");

        if dest_pass == pass {
            return Ok(SocketAddr::new(addr.ip(), dest_port));
        }
    }
}

fn display_code(socket: &UdpSocket) -> Result<u16> {
    let port = socket.local_addr()?.port();
    let pass = utils::rand_range(0, 255);
    let code = utils::encode(port, pass);
    ui::terminal::print_code(&code)?;

    Ok(pass)
}
