use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

pub fn highlight_line(line: &str, lang: &str) -> Result<String, Box<dyn std::error::Error>> {
    let syntax = SYNTAX_SET.find_syntax_by_extension(lang).unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(
        syntax,
        &THEME_SET.themes["base16-ocean.dark"],
    );

    let regions: Vec<(Style, &str)> = highlighter
        .highlight_line(line, &SYNTAX_SET)?;

    Ok(format!("{}", as_24_bit_terminal_escaped(&regions[..], true)))
}
