use anyhow::Result;
use std::io::stdout;
use crossterm::{
    cursor::{MoveLeft, MoveUp},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use icore::message::Message;

pub fn print_progress(progress: u8, current_line: &mut Option<Message>) -> Result<()> {
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
