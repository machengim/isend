mod cli;
mod network;
mod receiver;
mod sender;
mod utils;
mod ui;


fn main() {
    match cli::parse_args(){
        cli::Argument::S(s) => sender::launch(&cli::Argument::S(s)),
        cli::Argument::R(r) => receiver::launch(&cli::Argument::R(r)),
    }
}