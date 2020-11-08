use anyhow::{anyhow, Result};
use log::{debug, info};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::Mutex;
use crate::icore::message::{self, Message};

pub struct Typer {
    rx: Receiver<Message>,
    tx: Sender<String>,
}

lazy_static::lazy_static! {
    pub static ref TYPER: Mutex<Option<Typer>> = Mutex::new(None);
}

// Create a duplex channel between cli and icore-message.
pub fn init_channel() {
    info!("Function called: init_channel()");
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    message::init_msg_pipe(rx2, tx1);

    let mut typer = TYPER.lock().unwrap();
    *typer = Some(Typer {rx: rx1, tx: tx2});
    // Use drop() to unlock the mutex.
    std::mem::drop(typer);

    std::thread::spawn(|| {
        listen_msg();
    });
}

// listen on the channel continuously to get message from icore.
fn listen_msg() {
    let typer = TYPER.lock().unwrap();
    if typer.is_none() {
        // End the process if no typer found.
        print_fatal(&format!("Typer not initialized"));
    }

    let rx = &typer.as_ref().unwrap().rx;
    loop {
        debug!("Listen msg loop starts..");
        match rx.recv() {
            Ok(Message::Done)=> {break;},
            Ok(Message::Status(s)) => {print_status(&s);},
            Ok(Message::Progress(p)) => {},
            Ok(Message::Error(e)) => {print_error(&e);},
            Ok(Message::Fatal(f)) => {print_fatal(&f);}
            Ok(Message::Prompt(p)) => {},
            Err(e) => eprintln!("{}", e),
        }
    }
}



fn print_error(msg: &String) {
    eprintln!("Error: {}", msg);
}

// Notice that fatal error will terminate the whole process.
fn print_fatal(msg: &String) {
    eprintln!("Fatal Error: {}", msg);
    std::process::exit(1);
}

fn print_prompt(msg: &String) {
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_line(&mut input) {
        print_error(&format!("when reading input: {}", e));
    }

    let typer = TYPER.lock().unwrap();
    if typer.is_none() {
        print_fatal(&format!("Typer not initialized"));
    }

    let tx = &typer.as_ref().unwrap().tx;
    if let Err(e) = tx.send(input) {
        print_error(&format!("when sending user input to model: {}", e));
    }
}

fn print_status(msg: &String) {
    println!("{}", msg);
}