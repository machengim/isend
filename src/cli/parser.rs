use anyhow::anyhow;
use anyhow::Result;
use clap::{ArgMatches, Values};
use icore::arg::{Arg, OverwriteStrategy, SendArg, RecvArg};
use std::path::PathBuf;

pub fn parse_input(m: &ArgMatches) -> Result<Arg> {
    let arg = match (m.occurrences_of("send"), m.occurrences_of("receive")) {
        (1, 0) => parse_send_arg(m)?,
        (0, 1) => parse_recv_arg(m)?,
        _ => return Err(anyhow!("Unknow client type")),
    };

    Ok(arg)
}

fn parse_send_arg(m: &ArgMatches) -> Result<Arg> {
    let send_arg = SendArg {
        expire: parse_expire(m),
        files: parse_sending_files(m),
        msg: parse_msg(m),
        password: parse_password(m),
        port: parse_port(m),
        shutdown: parse_shutdown(m),
    };

    if send_arg.msg.is_some() || send_arg.files.is_some() {
        Ok(Arg::S(send_arg))
    } else {
        Err(anyhow!("Invalid arguments format"))
    }
}

fn parse_recv_arg(m: &ArgMatches) -> Result<Arg> {
    let code = parse_code(m)?;
    let dir = match parse_dir(m) {
        Some(d) => d,
        None => std::env::current_dir().expect("Cannot get current dir"),
    };

    let recv_arg = RecvArg {
        code,
        dir,
        expire: parse_expire(m),
        overwrite: parse_overwrite(m),
        password: parse_password(m),
        port: parse_port(m),
        shutdown: parse_shutdown(m),
    };

    Ok(Arg::R(recv_arg))
}

fn parse_password(m: &ArgMatches) -> Option<String> {
    if m.occurrences_of("password") > 0 {
        let mut pw = String::new();
        println!("Please enter the password: ");
        std::io::stdin().read_line(&mut pw)
            .expect("Failed to read line");
        let pw_str = pw.trim();
        if pw_str.len() > 0 && pw_str.len() <= 255{
            return Some(String::from(pw_str));
        }
    }

    None
}

fn parse_port(m: &ArgMatches) -> u16 {
    match m.value_of("port") {
        Some(p) => p.parse().expect("Cannot parse port number"),
        None => 0,
    }
}

fn parse_shutdown(m: &ArgMatches) -> bool {
    if m.occurrences_of("shutdown") > 0 {
        return true;
    }

    false
}

fn parse_expire(m: &ArgMatches) -> u8 {
    match m.value_of("expire") {
        Some(e) => e.parse().expect("Cannot parse expire minutes"),
        None => 0
    }
}

fn parse_msg(m: &ArgMatches) -> Option<String> {
    match m.value_of("msg") {
        Some(v) => Some(String::from(v)),
        None => None,
    }
}

fn parse_sending_files(m: &ArgMatches) -> Option<Vec<PathBuf>> {
    match m.values_of("INPUT") {
        Some(mut fs) => parse_files(&mut fs),
        None => None,
    }
}

fn parse_files(fs: &mut Values) -> Option<Vec<PathBuf>> {
    let mut files: Vec<_> = Vec::new();

    while let Some(f) = fs.next() { 
        let path = PathBuf::from(f);
        if path.is_file() || path.is_dir() {
            files.push(path);
        } else {
            eprintln!("Invalid path: {}", f);
        }
    }

    if files.len() > 0 { Some(files) } else { None }
}

fn parse_code(m: &ArgMatches) -> Result<String> {
    match m.values_of("INPUT") {
        Some(mut inputs) => parse_code_from_inputs(&mut inputs),
        None => Err(anyhow!("No code input")),
    }
}

fn parse_code_from_inputs(inputs: &mut Values) -> Result<String> {
    if inputs.len() != 1 {
        return Err(anyhow!("Only one code input allowed"));
    }

    match inputs.next() {
        Some(v) => Ok(String::from(v)),
        None => Err(anyhow!("Error parsing code from input")),
    }
}

fn parse_dir(m: &ArgMatches) -> Option<PathBuf> {
    match m.value_of("dir") {
        Some(dir) => {
            let path = PathBuf::from(dir);
            if path.is_dir() {Some(path)} else {None}
        },
        None => None,
    }
}

fn parse_overwrite(m: &ArgMatches) -> OverwriteStrategy {
    match m.value_of("overwrite") {
        Some("o") | Some("O") => OverwriteStrategy::Overwrite,
        Some("r") | Some("R") => OverwriteStrategy::Rename,
        Some("s") | Some("S") => OverwriteStrategy::Skip,
        _ => OverwriteStrategy::Ask,
    }
}