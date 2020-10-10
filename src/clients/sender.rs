use anyhow::Result;
use async_std::net::UdpSocket;
use std::net::SocketAddr;
use std::sync::mpsc;
use crate::{entities, ui, utils};

pub async fn launch(arg: &entities::SendArg) -> Result<SocketAddr>{
    let socket = UdpSocket::bind(("0.0.0.0", arg.port)).await?;
    let pass = display_code(&socket)?;

    let (tx, rx) = mpsc::channel::<bool>();
    ui::timer::start_timer((arg.expire * 60) as u64, rx);

    let dest = listen_upd(&socket, pass).await?;
    tx.send(true)?;

    Ok(dest)
}

fn display_code(socket: &UdpSocket) -> Result<u16> {
    let port = socket.local_addr()?.port();
    let pass = utils::rand_range(0, 255);
    let code = utils::encode(port, pass);
    ui::terminal::print_code(&code)?;

    Ok(pass)
}

async fn listen_upd(socket: &UdpSocket, pass: u16) -> Result<SocketAddr> {
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