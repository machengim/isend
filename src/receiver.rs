use crate::{cli, utils};
use crate::network::{get_socket_addr};


pub fn launch(arg: &cli::Argument) {
    let recv_arg: &cli::ReceiveArg = match arg {
        cli::Argument::R(r) => r,
        _ => panic!("Wrong input arguments"),
    };

    let dest_port = utils::hex_to_decimal(&recv_arg.code[..4]);
    println!("Dest port: {}", dest_port);

    let addr = get_socket_addr("0.0.0.0", recv_arg.port);

}