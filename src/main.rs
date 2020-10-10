mod clients;
mod entities;
mod ui;
mod utils;

#[async_std::main]
async fn main() {
    let send_arg = entities::SendArg::new();
    if let Err(e) = clients::sender::launch(&send_arg).await {
        eprintln!("Got error in sender: {}", e);
    }
}