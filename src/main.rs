mod cli;
mod sender;
mod utils;
mod ui;

fn main() {
    match cli::parse_args(){
        cli::Argument::S(s) => sender::launch(&s),
        _ => println!(" "),
    }
}