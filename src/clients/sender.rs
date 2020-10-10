use anyhow::Result;
use async_std::net::UdpSocket;
use rand::Rng;
use std::net::SocketAddr;
use std::sync::mpsc;
use crate::{entities, ui, utils};

pub async fn launch(arg: &entities::SendArg) -> Result<SocketAddr>{
    let socket = UdpSocket::bind(("0.0.0.0", arg.port)).await?;
    let pass = encode(&socket)?;

    let (tx, rx) = mpsc::channel::<bool>();
    ui::timer::start_timer((arg.expire * 60) as u64, rx);

    let mut buf = [0; 6];
    loop {
        let (_, addr) = socket.recv_from(&mut buf).await?;
        let (dest_port, dest_pass) = decode(std::str::from_utf8(&buf)?)
            .expect("Cannot parse code.");

        if dest_pass == pass {
            tx.send(true)?;
            return Ok(SocketAddr::new(addr.ip(), dest_port));
        }
    }
}

fn encode(socket: &UdpSocket) -> Result<u16> {
    let port = socket.local_addr()?.port();
    let pass = rand::thread_rng().gen_range(0, 256);
    let code = format!("{}{}", utils::dec_to_hex(port, 4), utils::dec_to_hex(pass, 2));

    print!("Your code is: \t");
    ui::terminal::print_color_text(&code)?;
    println!("");

    Ok(pass)
}

fn decode(code: &str) -> Option<(u16, u16)> {
    if code.len() != 6 || !utils::validate_hex_str(code) {
        println!("Invalid code string: {}", code);
        return None;
    }

    let port = utils::hex_to_decimal(&code[..4]);
    let pass = utils::hex_to_decimal(&code[4..]);

    Some((port, pass))
}