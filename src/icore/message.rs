use anyhow::{Result, anyhow};
use log::{debug, info};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub enum Message {
    Done,
    Error(String),
    Fatal(String),          // Leads to the termination of process.
    Progress(u8),
    Prompt(String),         // This one requires user input.
    Status(String),
    Time(u64),             
}

// a channel struct holds the
pub struct MsgPipe {
    rx: Receiver<String>,
    tx: Sender<Message>,
}

lazy_static::lazy_static!{
    static ref PIPE: Mutex<Option<MsgPipe>> = Mutex::new(None); 
}

// This function should be invoked by typer to register the channels.
pub fn init_msg_pipe(rx: Receiver<String>, tx: Sender<Message>) {
    let mut pipe = PIPE.lock().unwrap();
    *pipe = Some(MsgPipe{rx, tx});
    debug!("Message pipe created");
}

// public interface to send message to UI.
// The errors should be handled here since no other way to display them.
pub fn send_msg(msg: Message) {
    let pipe = PIPE.lock().unwrap();
    if pipe.is_none() {
        eprintln!("Fatal Error: message pipe is not initialized");
        std::process::exit(1);
    }

    let tx = &pipe.as_ref().unwrap().tx;
    if let Err(e) = tx.send(msg) {
        eprintln!("Cannot send message to UI: {}", e);
    }
}

pub fn send_prompt(msg: Message) -> String {
    if let Message::Prompt(_) = msg.clone() {
        send_msg(msg);
        match recv_input() {
            Ok(s) => return s,
            Err(e) => send_msg(Message::Error(format!("in receiving user input: {}", e))),
        }
    }

    String::new()
}

fn recv_input() -> Result<String> {
    let pipe = PIPE.lock().unwrap();
    if pipe.is_none() {
        eprintln!("Fatal Error: message pipe is not initialized");
        std::process::exit(1);
    }

    let rx = &pipe.as_ref().unwrap().rx;
    Ok(rx.recv()?)
}