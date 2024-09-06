use crossterm::execute;
use std::io::{stdout, Error};

pub fn display_scrollbar(
    offset: usize,
    total: usize,
    top: usize,
    height: usize,
    left: usize,
) -> Result<(), Error> {
    let carret_position = (offset as f64 / total as f64) * height as f64;

    for i in 0..height {
        let char = if i == carret_position as usize {
            '█'
        } else {
            '░'
        };
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(left as u16, (top + i) as u16),
            crossterm::style::Print(char),
        )?;
    }
    Ok(())
}

#[test]
fn display_scrollbar_test() {
    let result = display_scrollbar(0, 10, 0, 10, 0);
    assert!(result.is_ok());
}
