mod cli;
mod clients;
mod entities;
mod ui;
mod utils;

fn main() {
    match cli::read_input() {
        entities::Argument::S(_) => println!("sender"),
        entities::Argument::R(_) => println!("receiver"),
    }
}
