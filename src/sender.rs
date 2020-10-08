use std::net::UdpSocket;
use std::sync::mpsc;
use rand::Rng;
use crate::{cli, utils, ui::timer};

pub fn launch_udp(arg: &cli::SendArg) -> std::io::Result<()> {
    let udp_socket: UdpSocket = UdpSocket::bind("0.0.0.0:0")?;
    let port = udp_socket.local_addr()?
        .port();

    let pass = rand::thread_rng().gen_range(10, 100);
    let code = format!("{}{}", utils::port_to_hex(port), pass);
    println!("code is {}", code);

    let expire = arg.expire;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        timer::start_timer((expire * 60) as u64, rx);
    });

    let mut buf = [0; 6];
    while let Ok((n, addr)) = udp_socket.recv_from(&mut buf) {
        println!("{} bytes response from {:?}", n, addr);
        // TODO: Check whether the code is valid.
        tx.send(true);
    }

    Ok(())
}