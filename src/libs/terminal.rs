use std::cmp::{max, min};
use std::io::{stdout, Error};
use std::sync::atomic::{AtomicUsize, Ordering};

use crossterm::execute;

static SCREEN_HEIGHT: AtomicUsize = AtomicUsize::new(0);
static SCREEN_WIDTH: AtomicUsize = AtomicUsize::new(0);

pub fn print_at(x: u16, y: u16, text: &str) -> Result<(), Error> {
    let x = max(min(x, screen_width() as u16), 0);
    let y = max(min(y, screen_height() as u16), 0);
    execute!(
        stdout(),
        crossterm::cursor::MoveTo(x, y),
        crossterm::style::Print(text)
    )
}

pub fn hide_cursor() -> Result<(), Error> {
    execute!(stdout(), crossterm::cursor::Hide)
}

pub fn clear_lines(lines: &[u16]) -> Result<(), Error> {
    for y in lines.iter() {
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(0, *y),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
        )?;
    }
    Ok(())
}

pub fn enter_alternate_screen() -> Result<(), Error> {
    execute!(stdout(), crossterm::terminal::EnterAlternateScreen)
}

pub fn leave_alternate_screen() -> Result<(), Error> {
    execute!(stdout(), crossterm::terminal::LeaveAlternateScreen)
}

pub fn clear_screen() -> Result<(), Error> {
    execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))
}

pub fn enable_raw_mode() -> Result<(), Error> {
    crossterm::terminal::enable_raw_mode()
}

pub fn disable_raw_mode() -> Result<(), Error> {
    crossterm::terminal::disable_raw_mode()
}

pub fn get_screen_size() -> Result<(), std::io::Error> {
    let (width, height) = crossterm::terminal::size()?;
    SCREEN_WIDTH.store(width as usize, Ordering::SeqCst);
    SCREEN_HEIGHT.store(height as usize, Ordering::SeqCst);
    Ok(())
}

pub fn set_screen_size(width: usize, height: usize) {
    SCREEN_WIDTH.store(width, Ordering::SeqCst);
    SCREEN_HEIGHT.store(height, Ordering::SeqCst);
}

pub fn screen_width() -> usize {
    SCREEN_WIDTH.load(Ordering::SeqCst)
}

pub fn screen_height() -> usize {
    SCREEN_HEIGHT.load(Ordering::SeqCst)
}

// debug macro
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            print_at(0, screen_height() as u16 - 1, &format!($($arg)*)).unwrap();
        }
    };
}
