use anyhow::Result;
use async_std::net::TcpListener;
use async_std::prelude::*;
use std::sync::mpsc;
use std::{thread, time::Duration};
use crate::{entities, utils};

pub async fn launch(arg: &entities::RecvArg) -> Result<()> {
    println!("{:?}", arg);
    let dest_code = match &arg.code{
        Some(c) => c,
        None => panic!("Unknown code input."),
    };

    let tcp_socket = TcpListener::bind(("0.0.0.0", arg.port)).await?;
    let tcp_port = tcp_socket.local_addr()?.port();
    let retry = arg.retry;
    let tx = start_udp(dest_code, tcp_port, retry);

    let (mut stream, addr) = tcp_socket.accept().await?;
    tx.send(true)?;

    let mut buf = [0u8; 16];
    let _ = stream.read(&mut buf).await?;
    println!("{:?}{:?}", &buf, &addr);
    
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
