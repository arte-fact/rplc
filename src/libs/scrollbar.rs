use std::cmp::min;
use std::io::Error;

use super::terminal::print_at;

pub fn display_scrollbar(
    offset: usize,
    total: usize,
    top: usize,
    height: usize,
    left: usize,
) -> Result<(), Error> {
    let carret_position = min(
        (((offset as f64) / (total as f64 - height as f64)) * height as f64) as usize,
        height as usize - 1,
    );

    for i in 0..height {
        let char = if i == carret_position as usize {
            '█'
        } else {
            '░'
        };
        print_at(left as u16, (top + i) as u16, &char.to_string())?;
    }
    Ok(())
}

#[test]
fn display_scrollbar_test() {
    let result = display_scrollbar(0, 10, 0, 10, 0);
    assert!(result.is_ok());
}
