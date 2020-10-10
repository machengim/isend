mod clients;
mod entities;
mod ui;
mod utils;
use async_std::task::block_on;
use std::thread::spawn;

fn main() {
    let sender_handler = spawn(|| {
        block_on(start_sender());
    });

    let receiver_handler = spawn(||{
        block_on(start_receiver());
    });

    sender_handler.join().unwrap();
    receiver_handler.join().unwrap();
}

// The following functions are used for testing.
async fn start_sender() {
    let send_arg = entities::SendArg::new();
    if let Err(e) = clients::sender::launch(&send_arg).await {
        eprintln!("Got error in sender: {}", e);
    }
}

async fn start_receiver() {
    let recv_arg = entities::RecvArg::new();
    if let Err(e) = clients::receiver::launch(&recv_arg).await {
        eprintln!("Got error in sender: {}", e);
    }
}