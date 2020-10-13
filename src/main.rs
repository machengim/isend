mod cli;
mod clients;
mod arguments;
mod protocol;
mod ui;
mod utils;
use arguments::Argument::{S, R};
use clients::{sender, receiver};

#[async_std::main]
async fn main(){
    match cli::read_input() {
        S(s) => sender::launch(&s).await.unwrap(),
        R(r) => receiver::launch(&r).await.unwrap(),
    }
}
