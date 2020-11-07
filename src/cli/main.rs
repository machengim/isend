mod parser;
use clap::{App, load_yaml};
use icore::arg::Arg;
use icore::client::{Sender, Receiver};
use icore::message::Message;
use std::sync::mpsc;
use std::time::Duration;
use std::io::stdout;
use crossterm::{
    cursor::{MoveLeft, MoveUp},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

#[async_std::main]
async fn main() {
    let yaml = load_yaml!("cli.yaml");
    let m = App::from(yaml).get_matches();
    let (tx, rx) = mpsc::channel();

    let hanlde = std::thread::spawn(move || {
        if let Err(e) = receive_msg(rx) {
            println!("Get error: {}", e);
            std::process::exit(1);
        }
    });

    match parser::parse_input(&m) {
        Ok(Arg::S(send_arg)) => match Sender::launch(send_arg, tx).await {
            Ok(()) => {},
            Err(e) => eprint!("Error in sender: {}", e),
        },
        Ok(Arg::R(recv_arg)) => match Receiver::launch(recv_arg, tx).await {
            Ok(()) => {},
            Err(e) => eprint!("Error in receiver: {}", e),
        }
        Err(e) => eprintln!("Error in client: {}", e),
    }

    hanlde.join().unwrap();
    println!("Task done.");
}

fn receive_msg(rx: mpsc::Receiver<Message>) -> anyhow::Result<()> {
    let mut current_line: Option<Message> = None;

    loop {
        let mut progress = 0u8;

        while let Ok(msg) = rx.try_recv() {
            match msg {
                Message::Done => {
                    print_progress(progress, &mut current_line)?;
                    return Ok(());
                },
                Message::Error(e) => {
                    print_progress(progress, &mut current_line)?;
                    return Err(anyhow::anyhow!(e));
                },
                Message::Progress(p) => progress = p,
                Message::Status(s) => {
                    print_progress(progress, &mut current_line)?; 
                    println!("{}", s);
                    current_line = Some(Message::Status(s));
                },
            }
        }
        
        print_progress(progress, &mut current_line)?;

        std::thread::sleep(Duration::from_millis(200));
    }
}

fn print_progress(progress: u8, current_line: &mut Option<Message>) -> anyhow::Result<()> {
    if progress > 0 {
        if let Some(Message::Progress(_)) = current_line {
            stdout()
            .execute(MoveUp(1))?
            .execute(Clear(ClearType::CurrentLine))?
            .execute(MoveLeft(999))?;
        }

        println!("Progress: {}%", &progress);
        *current_line = Some(Message::Progress(0));
    }

    Ok(())
}
