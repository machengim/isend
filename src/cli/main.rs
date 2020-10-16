use anyhow::Result;
use async_std;
use icore::args::{SendArg, RecvArg};
use icore::sender;

#[async_std::main]
async fn main() -> Result<()> {
    let send_arg = SendArg::default();
    println!("{:?}", send_arg);
    sender::launch(send_arg).await?;

    Ok(())
}