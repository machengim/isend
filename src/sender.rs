use std::net::UdpSocket;
use rand::Rng;
use crate::{cli, utils};

pub fn launch_udp(arg: &cli::SendArg) -> std::io::Result<()> {
    let udp_socket: UdpSocket = UdpSocket::bind("0.0.0.0:0")?;
    let port = udp_socket.local_addr()
        .expect("Cannot bind local socket.")
        .port();

    let pass = rand::thread_rng().gen_range(10, 100);
    let code = format!("{}{}", utils::port_to_hex(port), pass);
    println!("code is {}", code);

    Ok(())
}