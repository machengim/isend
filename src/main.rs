mod cli;
mod sender;
mod utils;

fn main() {
    match cli::parse_args(){
        cli::Argument::S(s) => {sender::launch_udp(&s);}
        _ => println!(" "),
    }
}