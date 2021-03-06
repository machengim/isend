use super::message::{Message, send_msg, send_prompt};
use std::path::PathBuf;

pub enum Arg {
    S(SendArg),
    R(RecvArg),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OverwriteStrategy {
    Ask,
    Rename,
    Overwrite,
    Skip,
}

#[derive(Debug, Default)]
pub struct SendArg {
    pub expire: u8,
    pub files: Option<Vec<PathBuf>>,
    pub msg: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Default)]
pub struct RecvArg {
    pub expire: u8,
    pub dir: PathBuf,
    pub overwrite: OverwriteStrategy,
    pub password: Option<String>,
    pub code: u16,  // Port number
}

impl Default for OverwriteStrategy {
    fn default() -> Self {
        OverwriteStrategy::Ask
    }
}

impl OverwriteStrategy {
    // Ask the user for an overwrite strategy.
    // Note that 'ask' is not in the options but still used as default.
    pub fn ask() -> Self {
        let input = send_prompt(Message::Prompt(
            "Please choose: overwrite(o) | rename(r) | skip (s): ".to_string()));

        match input.trim() {
            "o" | "O" => return OverwriteStrategy::Overwrite,
            "r" | "R" => return OverwriteStrategy::Rename,
            "s" | "S" => return OverwriteStrategy::Skip,
            _ => {
                send_msg(Message::Status(format!("Unknown overwrite strategy chose")));
            }
        }
        
        OverwriteStrategy::Ask
    }
}
