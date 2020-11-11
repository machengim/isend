use anyhow::Result;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Mutex};

#[derive(Clone, Debug)]
pub enum Message {
    Done,
    Error(String),
    FileEnd,                // Current file transmission finished.
    Fatal(String),          // Leads to the termination of process.
    Progress(String),
    Prompt(String),         // This one requires user input.
    Status(String),
    Time(u64),             
}

// a channel struct holds the
pub struct MsgPipe {
    rx: Receiver<String>,
    tx: Sender<Message>,
}

// Init a fake MsgPipe.
lazy_static::lazy_static!{
    static ref PIPE: Mutex<MsgPipe> = {
        let (_, rx) = mpsc::channel();
        let (tx, _) = mpsc::channel();

        Mutex::new(MsgPipe{rx, tx})
    };
}

// This function should be invoked by typer to register the channels.
pub fn launch(rx: Receiver<String>, tx: Sender<Message>) {
    let mut pipe = PIPE.lock().unwrap();
    *pipe = MsgPipe{rx, tx};
    log::debug!("Message pipe created");
}

// public interface to send message to UI.
// The errors should be handled here since no other way to display them.
pub fn send_msg(msg: Message) {
    let pipe = PIPE.lock().unwrap();
    let tx = &pipe.tx;

    if let Err(e) = tx.send(msg) {
        eprintln!("Cannot send message to UI: {}", e);
    }
}

// Send prompt message to stdout and requires user input.
pub fn send_prompt(msg: Message) -> String {
    if let Message::Prompt(_) = msg.clone() {
        send_msg(msg);
        match recv_input() {
            Ok(s) => return s,
            Err(e) => send_msg(Message::Error(format!("in receiving user input: {}", e))),
        }
    } else {
        send_msg(Message::Error(format!("wrong format in sending prompt")));
    }

    String::new()
}

// Receive user input.
fn recv_input() -> Result<String> {
    let pipe = PIPE.lock().unwrap();
    let rx = &pipe.rx;

    Ok(rx.recv()?)
}
