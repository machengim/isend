use crate::{cli, utils};
use crate::network::{self, SocketType};


pub fn launch(arg: &cli::Argument) {
    let recv_arg: &cli::ReceiveArg = match arg {
        cli::Argument::R(r) => r,
        _ => panic!("Wrong input arguments"),
    };

    let addr = network::get_socket_addr("0.0.0.0", recv_arg.port);
    let tcp_socket = network::bind_socket(&addr, SocketType::TCP)
        .expect("Cannot bind TCP socket");

    let dest_port = utils::hex_to_decimal(&recv_arg.code[..4]);
    let addr = network::get_socket_addr("0.0.0.0", dest_port);
    let udp_socket = network::bind_socket(&addr, SocketType::UDP)
        .expect("Cannot bind UDP socket.");
    let udp = udp_socket.unwrap_udp()
        .expect("Cannot read UDP socket.");

    // TODO: generate contents to send to the UDP server.
    network::send_broadcast(&udp, dest_port);
}
