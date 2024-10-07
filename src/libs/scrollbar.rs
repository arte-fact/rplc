use std::cmp::min;
use std::io::Error;

use crossterm::style::Stylize;

use super::terminal::print_at;

pub fn display_scrollbar(
    offset: usize,
    total: usize,
    top: usize,
    height: usize,
    left: usize,
) -> Result<(), Error> {
    let carret_position = min(
        (((offset as f64) / (total as f64 - height as f64)) * (height - 3) as f64) as usize + 1,
        height as usize,
    );

    // print arrows
    for i in 0..height {
        let char = if i == carret_position as usize {
            &"█".dark_grey().to_string()
        } else {
            &"│".bold().dark_grey().to_string()
        };
        print_at(left as u16, (top + i) as u16, &char.to_string())?;
    }

    print_at(left as u16, top as u16, &"▲".dark_grey().to_string())?;
    print_at(left as u16, (top + height - 1) as u16, &"▼".dark_grey().to_string())?;

    Ok(())
}

#[test]
fn display_scrollbar_test() {
    let result = display_scrollbar(0, 10, 0, 10, 0);
    assert!(result.is_ok());
}
