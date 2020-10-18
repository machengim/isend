mod parser;

use anyhow::Result;
use clap::{App, load_yaml};
use icore::arg::Arg;
use icore::client::{Sender, Receiver};

#[async_std::main]
async fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yaml");
    let m = App::from(yaml).get_matches();

    match parser::parse_input(&m)? {
        Arg::S(send_arg) => match Sender::launch(send_arg).await {
            Ok(()) => {},
            Err(e) => eprint!("Error in sender: {}", e),
        },
        Arg::R(recv_arg) => match Receiver::launch(recv_arg).await {
            Ok(()) => {},
            Err(e) => eprint!("Error in receiver: {}", e),
        }
    }

    Ok(())
}