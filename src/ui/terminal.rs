use anyhow::Result;
use std::io::stdout;
use crossterm::{
    cursor::MoveLeft,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

pub fn refresh_line(s: &str) -> Result<()> {
    stdout()
        .execute(Clear(ClearType::CurrentLine))?
        .execute(MoveLeft(999))?
        .execute(Print(s))?;

    Ok(())
}

pub fn print_code(code: &str) -> Result<()> {
    print!("Your code is: \t");
    print_color_text(code)?;
    println!("");

    Ok(())
}

pub fn print_color_text(s: &str) -> Result<()> {
    stdout()
        .execute(SetForegroundColor(Color::White))?
        .execute(SetBackgroundColor(Color::Blue))?
        .execute(Print(s))?
        .execute(ResetColor)?;

    Ok(())
}