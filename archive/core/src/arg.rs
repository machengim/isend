use std::path::PathBuf;
use std::sync::mpsc::Sender;
use crate::message::Message;

pub enum Arg {
    S(SendArg),
    R(RecvArg),
}

#[derive(Clone, Copy, Debug)]
pub enum OverwriteStrategy {
    Ask,
    Rename,
    Overwrite,
    Skip,
}

impl Default for OverwriteStrategy {
    fn default() -> Self {
        OverwriteStrategy::Ask
    }
}

impl OverwriteStrategy {
    pub fn ask(tx: &Sender<Message>) -> Self {
        //println!("Please choose: overwrite(o) | rename(r) | skip (s): ");
        tx.send(Message::Status(format!("Please choose: overwrite(o) | rename(r) | skip (s): "))).unwrap();
        let mut input = String::new();
        if let Ok(_) = std::io::stdin().read_line(&mut input) {
            match input.trim() {
                "o" | "O" => return OverwriteStrategy::Overwrite,
                "r" | "R" => return OverwriteStrategy::Rename,
                "s" | "S" => return OverwriteStrategy::Skip,
                _ => {
                    //println!("Unknown overwrite strategy chose");
                    tx.send(Message::Status(format!("Unknown overwrite strategy chosen"))).unwrap();
                }
            }
        }
        
        OverwriteStrategy::Ask
    }
}

#[derive(Debug, Default)]
pub struct SendArg {
    pub expire: u8,
    pub files: Option<Vec<PathBuf>>,
    pub msg: Option<String>,
    pub password: Option<String>,
    pub port: u16,
    pub shutdown: bool,
}

#[derive(Debug, Default)]
pub struct RecvArg {
    pub code: String,
    pub expire: u8,
    pub dir: PathBuf,
    pub overwrite: OverwriteStrategy,
    pub password: Option<String>,
    pub port: u16,
    pub shutdown: bool,
}