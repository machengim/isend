mod parser;
use clap::{App, load_yaml};
use icore::arg::Arg;
use icore::client::{Sender, Receiver};
use std::time::{Instant, Duration};

#[async_std::main]
async fn main() {
    let yaml = load_yaml!("cli.yaml");
    let m = App::from(yaml).get_matches();

    let start = Instant::now();

    match parser::parse_input(&m) {
        Ok(Arg::S(send_arg)) => match Sender::launch(send_arg).await {
            Ok(()) => {},
            Err(e) => eprint!("Error in sender: {}", e),
        },
        Ok(Arg::R(recv_arg)) => match Receiver::launch(recv_arg).await {
            Ok(()) => {},
            Err(e) => eprint!("Error in receiver: {}", e),
        }
        Err(e) => eprintln!("Error in client: {}", e),
    }

    println!("Task done in {} seconds", start.elapsed().as_secs() );
}