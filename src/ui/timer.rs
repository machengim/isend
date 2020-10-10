use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::thread::{spawn, sleep};
use super::terminal;

pub fn start_timer(total: u64, rx: mpsc::Receiver<bool>) {
    spawn(move || {
        run_timer(total, rx);
    });
}

fn run_timer(total: u64, rx: mpsc::Receiver<bool>) {
    let start = Instant::now();
    
    loop {
        sleep(Duration::from_secs(1));

        if let Ok(true) = rx.try_recv() {
            println!("Get connection.");
            return;
        }

        let rest = total - start.elapsed().as_secs();
        if rest <= 0 {
            terminal::refresh_line("No connection got in time.\n")
                .expect("Cannot refresh line on terminal.");
            std::process::exit(0);
        }

        let time_text = get_time_text(rest);
        terminal::refresh_line(&time_text)
            .expect("Cannot refresh timer on terminal.");
    }
}

fn get_time_text(t: u64) -> String {
    let min = t / 60;
    let sec = t % 60;
    let sec_str = if sec < 10 { ["0", &sec.to_string()].concat() } else { sec.to_string() };

    format!("Awaiting response.. \t {}:{}", min, sec_str)
}