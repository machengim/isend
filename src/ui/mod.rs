pub mod timer;

use std::io::stdout;
use crossterm::{
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand, Result,
};

pub fn print_color_text(s: &str) -> Result<()>{
    stdout()
        .execute(SetForegroundColor(Color::White))?
        .execute(SetBackgroundColor(Color::Blue))?
        .execute(Print(s))?
        .execute(ResetColor)?
        .execute(Print('\n'))?;

    Ok(())
}