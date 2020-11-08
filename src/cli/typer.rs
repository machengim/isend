use anyhow::{anyhow, Result};
use log::{debug, info};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::Mutex;
use crate::icore::message::{self, Message};

enum LineType {
    Progress,
    Text,
    Time,
}

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
        match rx.recv() {
            Ok(Message::Done)=> print_done(),
            Ok(Message::Status(s)) => {print_status(&s);},
            Ok(Message::Progress(p)) => {},
            Ok(Message::Error(e)) => {print_error(&e);},
            Ok(Message::Fatal(f)) => {print_fatal(&f);}
            Ok(Message::Prompt(p)) => {},
            Ok(Message::Time(t)) => print_time(t),
            Err(e) => eprintln!("{}", e),
        }
    }
}

fn print_done() {
    println!("Task done");
    std::process::exit(0);
}

fn print_error(e: &String) {
    eprintln!("Error: {}", e);
}

// Notice that fatal error will terminate the whole process.
fn print_fatal(f: &String) {
    eprintln!("Fatal Error: {}", f);
    std::process::exit(1);
}

// Ask the user to input and send the result to the channel.
fn print_prompt(p: &String) {
    println!("{}", p);
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

fn print_status(s: &String) {
    println!("{}", s);
}

fn print_time(t: u64) {
    let time_str = format!("{}:{}", t / 60, t % 60);
    println!("Time left: {}", time_str);
}