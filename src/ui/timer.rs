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
        if let Ok(s) = tx.try_recv() {
            if s { return Ok(Status::Success); }
        }
        let text = timer_text(total - start.elapsed().as_secs());

        stdout()
            .execute(Clear(ClearType::CurrentLine))?
            .execute(MoveLeft(999))?
            .execute(Print(text))?;
    }

    stdout()
        .execute(Clear(ClearType::CurrentLine))?
        .execute(MoveLeft(999))?;

    Ok(Status::Timeout)
}

fn timer_text(rest: u64) -> String {
    let min = rest / 60;
    let sec = rest % 60;
    let sec_str = if sec >= 10 {sec.to_string()} else {format!("0{}", sec)};

    format!("Awating response {}:{}", min, sec_str)
}