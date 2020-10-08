use std::net::UdpSocket;
use std::sync::mpsc;
use rand::Rng;
use crate::{cli, utils, ui, ui::timer};

struct Udp {
    addr: UdpSocket,
    port: u16,
}

pub fn launch(arg: &cli::SendArg) {
    let Udp{addr: udp_socket, port} = match bind_udp(arg) {
        Ok(udp) => udp,
        Err(_) => {
            eprintln!("Cannot bind udp socket.\n");
            std::process::exit(1);
        }
    };

    let pass = rand::thread_rng().gen_range(10, 100);
    let code = format!("{}{}", utils::port_to_hex(port), pass);
    print!("Code is: ", );
    ui::print_color_text(&code).expect("Cannot print code");

    listen_udp(arg.expire, udp_socket);
}

fn bind_udp(arg: &cli::SendArg) -> std::io::Result<Udp> {
    let udp_socket: UdpSocket = UdpSocket::bind(("0.0.0.0", arg.port))?;

    let port = udp_socket.local_addr()?
        .port();

    Ok(Udp{addr: udp_socket, port})
}

fn listen_udp(expire: u16, udp_socket: UdpSocket) {
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
}