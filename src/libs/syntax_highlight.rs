use std::io::BufRead;
use syntect::easy::HighlightFile;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

pub fn highlight_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let ss = SyntaxSet::load_defaults_newlines();
    let mut ts = ThemeSet::load_defaults();
    ts.add_from_folder("assets/themes")?;

    let mut highlighter = HighlightFile::new(
        path,
        &ss,
        &ts.themes["Nord"],
    )?;

    let mut line = String::new();
    let mut result = String::new();
    while highlighter.reader.read_line(&mut line)? > 0 {
        {
            let regions: Vec<(Style, &str)> = highlighter
                .highlight_lines
                .highlight_line(&line, &ss)?;

            result.push_str( &format!("{}", as_24_bit_terminal_escaped(&regions[..], true)));
        }
        line.clear();
    }
    Ok(result)
}
