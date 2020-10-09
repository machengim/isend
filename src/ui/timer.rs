use std::io::stdout;
use std::sync::mpsc;
use std::time::Instant;
use crossterm::{
    style::Print,
    cursor::MoveLeft,
    terminal::{Clear, ClearType},
    ExecutableCommand, Result,
};


pub enum Status {
    Timeout,
    Success,
}

pub fn start_timer(total: u16) -> mpsc::Sender<bool> {
    let start = Instant::now();
    let (tx, rx) = mpsc::channel();

    if let Err(_) = stdout().execute(Print(timer_text(total as u64))) {
        panic!("Cannot display timer.\n");
    }

    std::thread::spawn(move || {
        match  run_timer(start, total as u64, rx) {
            Ok(Status::Timeout) => {
                println!("No connection established in time.\n");
                std::process::exit(0)
            },
            Ok(Status::Success) => println!("Start connecting...\n"),
            _ => panic!("Error happens when counting time"),
        }
    });

    tx
}

fn run_timer(start: Instant, total: u64, rx: mpsc::Receiver<bool>) -> Result<Status> {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        if let Ok(true) = rx.try_recv() {
            print_text("")?;
            return Ok(Status::Success)
        }

        let rest = total - start.elapsed().as_secs();
        if rest <= 0 {
            break;
        }

        let text = timer_text(rest);
        print_text(&text)?;
    }

    Ok(Status::Timeout)
}

fn timer_text(rest: u64) -> String {
    let min = rest / 60;
    let sec = rest % 60;
    let sec_str = if sec >= 10 {sec.to_string()} else {format!("0{}", sec)};

    format!("Awating response {}:{}", min, sec_str)
}

fn print_text(text: &str) -> Result<()> {
    stdout()
        .execute(Clear(ClearType::CurrentLine))?
        .execute(MoveLeft(999))?
        .execute(Print(text))?;

    Ok(())
}