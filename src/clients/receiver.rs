use anyhow::Result;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task::block_on;
use std::path::PathBuf;
use std::sync::mpsc;
use std::{thread, time::Duration};
use crate::{arguments, utils};
use crate::protocol::{Operation, Instruction};

pub struct Receiver {
    dir: PathBuf,
    overwrite: arguments::OverwriteStrategy,
    password: Option<String>,
    stream: Option<TcpStream>,
}

impl Receiver {
    pub async fn accept(arg: arguments::RecvArg, socket: &TcpListener) -> Result<Self> {
        // TODO: should use a timer to control the time to wait for connection.
        // The value retry is actually used for UDP broadcast.
        let mut receiver = Receiver::new(arg, None);
        for _ in 0..10 {
            let (mut stream, addr) = socket.accept().await?;

            let mut buf = [0u8; 6];
            stream.read(&mut buf).await?;
            println!("Get buffer: {:?}", &buf);

            let ins = Instruction::decode(&buf);
            receiver.stream = Some(stream);

            if receiver.validate_connection(&ins) {
                println!("Accept connection from {} ", addr);
                return Ok(receiver);
            } else {
                println!("Refused connection from {}", addr);
                // TEST RESPONSE !!
                receiver.stream.as_ref().unwrap().write_all(&[1, 2, 3, 4, 6, 7]).await?;
            }
        }

        eprintln!("Cannot establish a connection");
        std::process::exit(1);
    }

    fn new(arg: arguments::RecvArg, stream: Option<TcpStream>) -> Self {
        Receiver {
            dir: match arg.dir {
                Some(dir) => dir,
                None => std::env::current_dir()
                    .expect("Cannot get current working directory"),
            },
            overwrite: arg.overwrite,
            password: arg.password,
            stream,
        }
    }

    // TODO: check password before connection.
    fn validate_connection(&self, ins: &Instruction) -> bool {
        match ins.operation {
            Operation::ConnWithoutPass => self.password.is_none(),
            Operation::ConnWithPass =>  block_on(self.compare_pass(ins)).unwrap(),
            _ => false,
        }
    }

    async fn compare_pass(&self, ins: &Instruction) -> Result<bool> {
        if self.password.is_none() {
            return Ok(false);
        }

        let mut buf = vec![0u8; ins.length as usize];
        let mut stream = match &self.stream{
            Some(s) => s,
            None => {
                eprintln!("No TCP stream found");
                std::process::exit(1);
            }
        };

        stream.read(&mut buf).await?;
        println!("Get buffer: {:?}", &buf);

        Ok(self.password.as_ref().unwrap() == &String::from_utf8(buf).unwrap())
    }
}

pub async fn launch(arg: arguments::RecvArg) -> Result<()> {
    let dest_code = &arg.code;

    let tcp_socket = TcpListener::bind(("0.0.0.0", arg.port)).await?;
    let tcp_port = tcp_socket.local_addr()?.port();
    let tx = start_udp(dest_code, tcp_port, arg.retry);

    let receiver = Receiver::accept(arg, &tcp_socket).await?;
    tx.send(true)?;

    /*
    let mut buf = [0u8; 6];
    let _ = stream.read(&mut buf).await?;
    let ins = Instruction::decode(&buf);
    println!("{:?}", &ins); 
    */
    
    Ok(())
}

fn start_udp(dest_code: &str, tcp_port: u16, retry: u8) -> mpsc::Sender<bool> {
    let (udp_port, pass) = utils::decode(&dest_code)
        .expect("Cannot parse code info");
    let code = utils::encode(tcp_port, pass);
    let (tx, rx) = mpsc::channel::<bool>();

    thread::spawn(move || {
        if let Err(e) = send_udp_broadcast(udp_port, &code, retry, rx){
            eprintln!("Got an error when binding UDP socket: {}", e);
        }
    });

    tx
}

fn send_udp_broadcast(port: u16, code: &str, retry: u8, rx: mpsc::Receiver<bool>)
     -> Result<()> {

    let udp_socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    udp_socket.set_broadcast(true)?;
    
    for _ in 0..retry {
        thread::sleep(Duration::from_secs(5));
        if let Ok(true) = rx.try_recv() {
            return Ok(())
        }

        udp_socket.send_to(code.as_bytes(), ("255.255.255.255", port))?;
    }

    eprintln!("Cannot establish a connection.");
    std::process::exit(1);
}

