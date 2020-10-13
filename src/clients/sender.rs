use anyhow::Result;
use async_std::net::{UdpSocket, TcpStream};
use async_std::prelude::*;
use std::net::SocketAddr;
use std::sync::mpsc;
use crate::{arguments, ui, utils};

pub async fn launch(arg: &arguments::SendArg) -> Result<()>{
    let socket = UdpSocket::bind(("0.0.0.0", arg.port)).await?;
    let pass = display_code(&socket)?;

    let (tx, rx) = mpsc::channel::<bool>();
    ui::timer::start_timer((arg.expire * 60) as u64, rx);

    let dest = listen_upd(&socket, pass).await?;
    tx.send(true)?;
    let mut stream = TcpStream::connect(dest).await?;
    stream.write_all(b"Hello").await?;

    Ok(())
}

pub async fn listen_upd(socket: &UdpSocket, pass: u16) 
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
