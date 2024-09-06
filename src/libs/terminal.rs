use std::io::{stdout, Error};

use crossterm::execute;

pub fn print_at(x: u16, y: u16, text: &str) -> Result<(), Error> {
    execute!(
        stdout(),
        crossterm::cursor::MoveTo(x, y),
        crossterm::style::Print(text)
    )
}
