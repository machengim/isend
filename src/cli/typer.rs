use anyhow::{anyhow, Result};
use std::io::{self, Write};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::Mutex;
use crate::icore::message::{self, Message};

// Type of lines in terminal.
#[derive(Clone)]
enum LineType {
    Text,
    Time,
    Progress,
}

lazy_static::lazy_static! {
    static ref CURRENT_LINE_TYPE: Mutex<LineType> = Mutex::new(LineType::Text);
}

// Entry point of typer. 
// Create two channels to communicate between UI and model.
pub fn launch() {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    message::launch(rx2, tx1);

    std::thread::spawn(move || {
        listen_msg(rx1, tx2);
    });
}

// listen on the channel continuously to get message from icore.
fn listen_msg(rx: Receiver<Message>, tx: Sender<String>) {
    loop {
        match rx.recv() {
            Ok(Message::Done)=> print_done(),
            Ok(Message::Status(s)) => {print_status(&s);},
            Ok(Message::Progress(p)) => {},
            Ok(Message::Error(e)) => {print_error(&e);},
            Ok(Message::Fatal(f)) => {print_fatal(&f);}
            Ok(Message::Prompt(p)) => {print_prompt(&p, &tx)},
            Ok(Message::Time(t)) => print_time(t),
            Err(e) => eprintln!("{}", e),
        }
    }
}

fn change_current_line_type(t: LineType) {
    let mut current = CURRENT_LINE_TYPE.lock().unwrap();
    *current = t;
}

// Get the current line type.
fn get_current_line_type() -> LineType {
    let current = CURRENT_LINE_TYPE.lock().unwrap();

    current.clone()
}

// Check whether current line type is text or progress.
fn check_time_pg() -> bool {
    match get_current_line_type() {
        LineType::Text | LineType::Progress => true,
        _ => false,
    }
}

// If current line is time or progress, the next text line
// needs to remove these line.
fn check_for_text() {
    if check_time_pg() {
        print!("\r");
        flush();
    }
}

fn print_done() {
    check_for_text();
    println!("Task done");
    flush();

    // wait half a second so that all output finish.
    std::thread::sleep(std::time::Duration::from_secs(1));
    std::process::exit(0);
}

fn print_error(e: &String) {
    check_for_text();
    eprintln!("Error: {}", e);
    change_current_line_type(LineType::Text);
}

// Notice that fatal error will terminate the whole process.
fn print_fatal(f: &String) {
    check_for_text();
    eprintln!("Fatal Error: {}", f);
    std::process::exit(1);
}

// Ask the user to input and send the result to the channel.
fn print_prompt(p: &String, tx: &Sender<String>) {
    check_for_text();
    print!("{}", p);
    flush();
    
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_line(&mut input) {
        print_error(&format!("when reading input: {}", e));
    }

    if let Err(e) = tx.send(input) {
        print_error(&format!("when sending user input to model: {}", e));
    }

    change_current_line_type(LineType::Text);
}

fn print_status(s: &String) {
    check_for_text();
    println!("{}", s);
    change_current_line_type(LineType::Text);
}

fn print_time(t: u64) {
    let time_str = format!("{}:{}", t / 60, t % 60);
    print!("\rTime left: {}", time_str);
    flush();
    change_current_line_type(LineType::Time);
}

fn flush() {
    if let Err(e) = io::stdout().flush() {
        print_error(&format!("in stdout: {}", e));
    }
}