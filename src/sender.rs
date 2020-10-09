use crate::{cli, ui, utils};
use crate::network::{SocketType, bind_socket, get_socket_addr, listen_udp};


pub fn launch(arg: &cli::Argument) {
    let send_arg: &cli::SendArg = match arg {
        cli::Argument::S(s) => s,
        _ => panic!("Wrong input arguments."),
    };

    let addr = get_socket_addr("0.0.0.0", send_arg.port);
    let udp_socket = bind_socket(&addr, SocketType::UDP)
        .expect("Cannot bind udp socket");

    // Generate code for connection.
    // The first 4 are port, the rest 2 are password.
    let port = udp_socket.get_socket_port()
        .expect("Cannot read port from udp socket.");
    let password = print_code(port, 2);

    let tx = ui::timer::start_timer(send_arg.expire * 60);
    let dest_socket = listen_udp(&udp_socket, &password)
        .expect("Error happens when listening on UDP.");
    tx.send(true)
        .expect("Send message to channel failed.");
}

// length range: 0 ~ 8
fn print_code(port: u16, length: usize) -> String {
    let port_str = utils::decimal_to_hex(port as u32, 4);
    let pass_str = utils::generate_rand_hex_code(length);
    let code = format!("{}{}", port_str, pass_str);

    print!("Code is: ", );
    ui::print_color_text(&code)
        .expect("Cannot print code.");

    return pass_str;
}