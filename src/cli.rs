use clap::{App, load_yaml, ArgMatches, Values};
use crate::entities::{Argument, SendArg, RecvArg};
use std::process::exit;

pub fn read_input() -> Argument {
    let yaml = load_yaml!("../cli.yaml");
    let m = App::from(yaml).get_matches();

    parse_input(&m)
}

fn parse_input(m: &ArgMatches) -> Argument {
    match (m.occurrences_of("send"), m.occurrences_of("receive")) {
        (1, 0) => parse_sender(m),
        (0, 1) => parse_receiver(m),
        _ => {
            eprintln!("Unknow client type"); 
            exit(1)
        },
    }
}

fn parse_sender(m: &ArgMatches) -> Argument {
    let mut arg = SendArg::new();
    if let Some(mut inputs) = m.values_of("INPUT") {
        arg.files = parse_sending_files(&mut inputs);
    }
    if let Some(msg) = m.value_of("message") {
        arg.msg = Some(String::from(msg));
    }
    
    if arg.msg.is_none() && arg.files.is_none() {
        eprintln!("No files or message are specified");
        exit(1);
    }

    println!("{:?}", &arg);
    Argument::S(arg)
}

fn parse_receiver(m: &ArgMatches) -> Argument {
    let mut arg = RecvArg::new();
    if let Some(mut inputs) = m.values_of("INPUT") {
        arg.code = parse_code(&mut inputs);
    }

    println!("{:?}", &arg.code);

    Argument::R(arg)
}

fn parse_code(inputs: &mut Values) -> Option<String> {
    if inputs.len() > 1 {
        eprintln!("Only one code allowed");
        exit(1);
    }

    match inputs.next() {
        Some(c) => Some(String::from(c)),
        None => {
            eprintln!("No code found"); 
            exit(1)
        },
    }
}

fn parse_sending_files(fs: &mut Values) -> Option<Vec<String>> {
    if fs.len() <= 0 {
        return None;
    }

    let mut files: Vec<String> = Vec::new();
    while let Some(f) = fs.next() {
        if !std::path::Path::new(f).exists() {
            eprintln!("Invalid path found");
            exit(1);
        }

        files.push(String::from(f));
    }

    Some(files)
}