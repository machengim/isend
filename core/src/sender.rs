use anyhow::Result;
use async_std::net::{TcpStream, UdpSocket};
use async_std::prelude::*;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::path::PathBuf;
use crate::{args, instruction, utils, notify};
use crate::communication::Message;

pub struct Sender {
    id: u16,
    files: Option<Vec<PathBuf>>,
    msg: Option<String>,
    password: Option<String>,
    stream: TcpStream,
}

impl Sender {
    fn new(arg: &args::SendArg, stream: TcpStream) -> Sender{
        Sender {
            id: 0,
            files: arg.files.clone(),
            msg: arg.msg.clone(),
            password: arg.password.clone(),
            stream,
        }
    }

    async fn connect(&mut self) -> Result<()> {
        let mut ins = instruction::Instruction::conn_without_pass(self.id);

        match &self.password {
            Some(pw) => {
                let pass = pw.clone();
                self.send(&mut ins, Some(Box::new(pass.as_bytes()))).await?;
            },
            None => self.send(&mut ins, None).await?,
        }

        Ok(())
    }

    async fn send(&mut self, ins: &mut instruction::Instruction,
        content: Option<Box<&[u8]>>)
         -> Result<()> {
        match &content {
            Some(bytes) => {
                ins.buffer = true;
                ins.length = bytes.len() as u16;
            },
            None => ins.buffer = false,
        }

        let buf = ins.encode();
        self.stream.write_all(&buf).await?;

        if let Some(s) = content {
            self.stream.write_all(&s).await?;
        }
    
        Ok(())
    }
}

pub async fn launch(arg: args::SendArg) -> Result<()> {
    let udp = UdpSocket::bind(("0.0.0.0", arg.port)).await?;
    let port = udp.local_addr()?.port();
    let pass = utils::rand_range(0, 256);

    let code = utils::encode(port, pass);
    notify(Message::Status(code));

    let dest = listen_upd(&udp, pass).await?;
    let stream = TcpStream::connect(dest).await?;

    let mut sender = Sender::new(&arg, stream);
    sender.connect().await?;

    Ok(())
}

async fn listen_upd(socket: &UdpSocket, pass: u16) 
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

// Convert a 6 char string to a tupe of port and password.
fn parse_code(code: &str) -> Result<(u16, u16)> {
    if code.len() != 6 || !utils::validate_hex_str(code) {
        let err = Error::new(ErrorKind::Other, "Cannot parse code");
        return Err(anyhow::Error::new(err));
    }

    let port = utils::hex_to_dec(&code[..4]);
    let pass = utils::hex_to_dec(&code[4..]);

    Ok((port, pass))
}
