use std::io::stdout;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use crossterm::{
    style::Print,
    cursor::MoveLeft,
    terminal::{Clear, ClearType},
    ExecutableCommand, Result,
};

enum Status {
    Timeout,
    Success,
}

pub fn start_timer(total: u64, tx: Receiver<bool>) {
    let start = Instant::now();

    if let Err(_) = stdout().execute(Print(timer_text(total))) {
        eprintln!("Cannot start timer.\n");
        std::process::exit(1);
    }

    match run_timer(total, start, tx) {
        Ok(Status::Timeout) => {
            println!("No response got in specific duration.\n");
            std::process::exit(0);
        },
        Ok(Status::Success) => {
            println!("Start connecting...\n");
        }
        Err(_) => {
            eprintln!("Error happens when running timer.\n");
            std::process::exit(1);
        }
    }
}

fn run_timer(total: u64, start: Instant, tx: Receiver<bool>) -> Result<Status> {
    while start.elapsed().as_secs() < total {
        std::thread::sleep(std::time::Duration::from_secs(1));
        // If received message from main process then stop loop.
        if let Ok(true) = tx.try_recv() {
            print_text("")?;
            return Ok(Status::Success); 
        }
        let text = timer_text(total - start.elapsed().as_secs());
        print_text(&text)?;
    }

    print_text("")?;

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