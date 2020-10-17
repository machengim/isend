use anyhow::Result;
use async_std;
use icore::arg::{SendArg, RecvArg};
use icore::client;

#[async_std::main]
async fn main() -> Result<()> {
    let send_arg = SendArg::default();
    println!("{:?}", send_arg);
    client::Sender::launch(send_arg).await?;

    Ok(())
}