use std::vec::Vec;
use clap::{Arg, App, ArgMatches, Values};

pub enum Argument {
    S(SendArg),
    R(ReceiveArg),
}

#[derive(Debug)]
pub struct SendArg {
    expire: u8,
    length: u8,
    port: u16,
    msg: Option<String>,
    files: Option<Vec<String>>,
    shutdown: bool
}

#[derive(Debug)]
pub struct ReceiveArg {
    expire: u8,
    port: u16,
    dir: Option<String>,
    code: String,
    shutdown: bool,
    overwrite: bool,
}

impl SendArg {
    fn default() -> Self {
        SendArg {
            expire: 10,
            length: 6,
            port: 0,
            msg: None,
            files: None,
            shutdown: false
        }
    }

    fn new(m: &ArgMatches) -> Self {
        let mut args = SendArg::default();

        if let Some(s) = m.value_of("msg") {
            args.msg = Some(String::from(s));
        }

        // Note the multiple files are allowed.
        if let Some(mut inputs) = m.values_of("INPUT") {
            args.files = parse_files(&mut inputs);
        }

        // If files and message are both not specified, the process should exit.
        if args.msg.is_none() && args.files.is_none() {
            eprintln!("No files or message specified.");
            std::process::exit(1);
        }

        args
    }
}

impl ReceiveArg {
    fn default() -> Self {
        ReceiveArg {
            expire: 10,
            port: 0,
            dir: None,
            code: String::new(),
            shutdown: false,
            overwrite: false,
        }
    }

    fn new(m: &ArgMatches) -> Self {
        let mut args = ReceiveArg::default();

        if let Some(mut inputs) = m.values_of("INPUT") {
            args.code = parse_code(&mut inputs);
        }

        args
    }
}

fn parse_code(inputs: &mut Values) -> String {
    let code: String = match inputs.next() {
        Some(c) => String::from(c),
        None => {
            eprintln!("No code found in input.");
            std::process::exit(1);
        }
    };

    // Only one code allowed in input.
    if let Some(_) = inputs.next() {
        eprintln!("Only one code allowed in arguments.");
        std::process::exit(1);
    }

    code
}

fn parse_files(fs: &mut Values) -> Option<Vec<String>> {
    let mut files: Vec<String> = Vec::new();

    while let Some(f) = fs.next() {
        if ! std::path::Path::new(f).exists() {
            eprintln!("Invalid path: {}.", f);
            std::process::exit(1);
        }

        files.push(String::from(f));
    }

    if files.len() == 0 {
        return None;
    }

    Some(files)
}

pub fn parse_args() -> Argument {
    let m = prepare_arg_list();
    
    match (m.occurrences_of("s"), m.occurrences_of("r")) {
        (1, 0) => {
            return Argument::S(SendArg::new(&m));
        }
        (0, 1) => {
            return Argument::R(ReceiveArg::new(&m));
        }
        _ => {
            eprintln!("Invalid sides in arguments. Please check --help.");
            std::process::exit(1);
        }
    }

}

fn prepare_arg_list() -> ArgMatches {
    App::new("Insta-Share")
        .version("0.1.0")
        .author("Cheng Ma <machengiam@gmail.com>")
        .arg(Arg::new("s")
            .short('s')
            .long("send")
            .about("Indicate this is the sending side")
            .takes_value(false)
        )
        .arg(Arg::new("r")
            .short('r')
            .long("receive")
            .about("Indicate this is the receiving side")
            .takes_value(false)
        )
        .arg(Arg::new("msg")
            .short('m')
            .long("msg")
            .takes_value(true)
            .multiple(true)
            .about("The message to send")
            )
        // Note this INPUT could be files for the sender and code for the receiver.
        .arg(Arg::new("INPUT")
            .required(true)
            .multiple(true)
            .about("The files to send or code to receive")
            .index(1)
            )
        .get_matches()
}