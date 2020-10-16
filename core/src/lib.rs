pub mod args;
pub mod communication;
pub mod instruction;
pub mod sender;

mod utils;

#[macro_use]
extern crate lazy_static;

pub fn notify(msg: communication::Message) {
    println!("{:?}", msg);
}