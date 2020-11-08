mod cli;
mod icore;
mod logger;
use clap::{load_yaml, App};
use cli::parser::parse_input;
use icore::arg::{Arg, SendArg, RecvArg};
use icore::{receiver, sender};

#[async_std::main]
async fn main() {
    if let Err(e) = logger::init_log() {
        eprintln!("Error in logger: {}", e);
        std::process::exit(1);
    }

    let yaml = load_yaml!("cli/cli.yaml");
    let m = App::from(yaml).get_matches();

    match parse_input(&m) {
        Ok(Arg::R(r)) => start_receiver(r).await,
        Ok(Arg::S(s)) => start_sender(s).await,
        Err(e) => eprint!("{}", e),
    }
}

async fn start_sender(s: SendArg) {
    log::debug!("Get sender arg:\n{:?}", &s);

    if let Err(e) = sender::launch(s).await {
        eprintln!("Error in sender: {}", e);
        std::process::exit(1);
    }
}

async fn start_receiver(r: RecvArg) {
    log::debug!("Get receiver arg:\n{:?}", &r);

    if let Err(e) = receiver::launch(r).await {
        eprintln!("Error in receiver: {}", e);
        std::process::exit(1);
    }
}