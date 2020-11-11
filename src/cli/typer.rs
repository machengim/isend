use std::io::{self, Write};
use std::sync::mpsc::{self, Sender, Receiver};
use crate::icore::message::{self, Message};

enum LineType {
    Text,
    Time,
    Progress,
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
    // False if the next content needs to start a new line or just refresh the current.
    // Only the timer and progress are not fixed (true).
    let mut line = LineType::Text;    

    loop {
        match rx.recv() {
            Ok(Message::Done)=> print_done(&mut line),
            Ok(Message::Error(s)) => print_error(&s, &mut line),
            Ok(Message::Fatal(s)) => print_fatal(&s, &line),
            Ok(Message::FileEnd) =>  print_file_end(&mut line),
            Ok(Message::Progress(s)) => print_progress(&s, &mut line),
            Ok(Message::Prompt(s)) => print_prompt(&s, &tx, &mut line),
            Ok(Message::Status(s)) => print_status(&s, &mut line),
            Ok(Message::Time(time)) => print_time(&time, &mut line),
            Err(e) => eprintln!("{}", e),
        }
    }
}

fn print_done(line: &mut LineType) {
    check_line_type(line, true);
    println!("Task done");

    std::process::exit(0);
}

fn print_error(s: &String, line: &mut LineType) {
    check_line_type(line, true);
    eprintln!("Error: {}", s);
    *line = LineType::Text;
}

fn print_file_end(line: &mut LineType) {
    println!();

    *line = LineType::Text;
}

fn print_fatal(s: &String, line: &LineType) {
    check_line_type(line, true);
    eprintln!("Fatal error: {}\nProcess exit", s);
    std::process::exit(1);
}

fn print_progress(s: &String, line: &mut LineType) {
    check_line_type(line, false);
    print_flush(&format!("{}", s));

    *line = LineType::Progress;
}

fn print_prompt(s: &String, tx: &Sender<String>, line: &mut LineType) {
    check_line_type(line, true);
    print_flush(&format!("{}", s));
    
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_line(&mut input) {
        print_error(&format!("when reading input: {}", e), line);
    }

    if let Err(e) = tx.send(input) {
        print_error(&format!("when sending user input to model: {}", e), line);
    }

    *line = LineType::Text;
}

fn print_status(s: &String, line: &mut LineType) {
    match *line {
        LineType::Time => print!("\r"),
        LineType::Progress => println!(),
        _ => (),
    }

    println!("{}", s);

    *line = LineType::Text;
}

fn print_time(t: &u64, line: &mut LineType) {
    check_line_type(line, false);

    let time_str = format!("{}:{}", t / 60, t % 60);
    print_flush(&format!("Time left: {}", time_str));

    *line = LineType::Time;
}

fn print_flush(s: &String) {
    print!("{}", s);

    if let Err(e) = io::stdout().flush() {
        eprintln!("Cannot flush buffer to terminal: {}", e);
    }
}

// Prepare the line before printing contents.
// Clear current line or start a new line.
fn check_line_type(line: &LineType, new_line: bool) {
    match (is_refresh(line), new_line) {
        (true, true) => println!(),
        (true, false) => print!("\r"),
        _ => (),
    }
}

// check whether current line is refreshable.
fn is_refresh(line: &LineType) -> bool {
    match line {
        LineType::Text => false,
        _ => true,
    }
}
