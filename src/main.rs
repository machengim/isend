mod cli;
mod clients;
mod entities;
mod ui;
mod utils;
use async_std::task::block_on;

fn main(){
    match cli::read_input() {
        entities::Argument::S(s) => block_on(clients::sender::launch(&s)).unwrap(),
        entities::Argument::R(r) => block_on(clients::receiver::launch(&r)).unwrap(),
    }
}
