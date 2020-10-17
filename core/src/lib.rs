pub mod arg;
pub mod client;
pub mod communication;
pub mod instruction;

mod utils;

pub fn notify(msg: communication::Message) {
    println!("{:?}", msg);
}