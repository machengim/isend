use clap::{App, load_yaml, ArgMatches};
use std::process::exit;
use crate::arguments::{Argument, RecvArg, SendArg};

pub fn read_input() -> Argument {
    let yaml = load_yaml!("../cli.yaml");
    let m = App::from(yaml).get_matches();

    parse_input(&m)
}

fn parse_input(m: &ArgMatches) -> Argument {
    match (m.occurrences_of("send"), m.occurrences_of("receive")) {
        (1, 0) => Argument::S(SendArg::from_match(m)),
        (0, 1) => Argument::R(RecvArg::from_match(m)),
        _ => {
            eprintln!("Unknow client type"); 
            exit(1)
        },
    }
}