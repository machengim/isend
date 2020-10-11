use clap::{App, load_yaml, ArgMatches};
use crate::entities::{Argument, SendArg, RecvArg};

enum ClientType {
    Sender,
    Receiver,
    Unknown,
}

pub fn read_input() -> Argument {
    let yaml = load_yaml!("../cli.yaml");
    let m = App::from(yaml).get_matches();
    let arg = parse_input(&m);

    arg
}

fn parse_input(m: &ArgMatches) -> Argument {
    match get_client_type(m) {
        ClientType::Sender => parse_sender(m),
        ClientType::Receiver => parse_receiver(m),
        ClientType::Unknown => {
            eprintln!("Unknow client type. Please indicate it by -s or -r.");
            std::process::exit(1);
        }
    }
}

fn parse_sender(m: &ArgMatches) -> Argument {
    Argument::S(SendArg::new())
}

fn parse_receiver(m: &ArgMatches) -> Argument {
    Argument::R(RecvArg::new())
}

fn get_client_type(m: &ArgMatches) -> ClientType {
    match (m.occurrences_of("send"), m.occurrences_of("receive")) {
        (1, 0) => ClientType::Sender,
        (0, 1) => ClientType::Receiver,
        _ => ClientType::Unknown,
    }
}