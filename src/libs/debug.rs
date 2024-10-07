use std::fs;
use std::io::{BufWriter, Write};

use chrono::Local;

use super::terminal::{print_at, screen_height};

fn append_string_to_file(path: &str, data: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let mut file = BufWriter::new(file);

    file.write_all(&data.as_bytes())?;

    file.flush()?;

    Ok(())
}

pub fn log_message(message: &String) {
    let format = "%Y-%m-%d %H:%M:%S";
    let datetime = Local::now().format(format);
    let message = format!("[{}]: {}\n", datetime, message);
    tokio::spawn(async move {
        match append_string_to_file("rplc.log", &message) {
            Ok(_) => (),
            Err(e) => {
                print_at(0, screen_height() as u16, &e.to_string()).unwrap();
            }
        }
    });
}

pub mod log {
    #[macro_export]
    macro_rules! log {
        ($($arg:tt)*) => {
            $crate::libs::debug::log_message(&format!($($arg)*));
        }
    }
}
