use std::io::Error;

use crossterm::style::Stylize;

use super::terminal::{clear_lines, cursor_at, print_at, show_cursor};

#[derive(Debug, PartialEq, Default)]
pub struct QuerySplit {
    pub query: Option<String>,
    pub glob: Option<String>,
    pub search: Option<String>,
    pub replace: Option<String>,
}

impl QuerySplit {
    pub fn display_with_colors(&self) -> String {
        let mut display = String::new();
        if let Some(glob) = &self.glob {
            display.push_str(glob.to_string().stylize().bold().blue().to_string().as_str());
        }
        if let Some(search) = &self.search {
            display.push(' ');
            display.push_str(search.to_string().stylize().bold().yellow().to_string().as_str());
        }
        if let Some(replace) = &self.replace {
            display.push(' ');
            display.push_str(replace.to_string().stylize().bold().green().to_string().as_str());
        }
        display
    }

    pub fn len(&self) -> usize {
        match self.query {
            Some(ref query) => query.len(),
            None => 0,
        }
    }

    pub fn print(&self) -> Result<(), Error> {
        clear_lines(&[2])?;
        let content = self.display_with_colors();
        print_at(0, 2, &content)?;
        self.restore_cursor()
    }

    pub fn restore_cursor(&self) -> Result<(), Error> {
        cursor_at(self.len() as u16, 2)?;
        show_cursor()
    }
}

fn update_split_query(split_query: &mut QuerySplit, temp: &str) {
    if temp.is_empty() {
        return;
    }
    if split_query.glob.is_none() {
        split_query.glob = Some(temp.to_string());
    } else if split_query.search.is_none() {
        split_query.search = Some(temp.to_string());
    } else if split_query.replace.is_none() {
        split_query.replace = Some(temp.to_string());
    }
}

pub fn split_query(query: &str) -> QuerySplit {
    let mut temp = String::new();
    let mut quote_char = None;
    let mut split_query = QuerySplit::default();

    for char in query.chars() {
        if ['"', '\''].contains(&char) {
            if quote_char.is_none() {
                quote_char = Some(char);
            } else if quote_char == Some(char) {
                quote_char = None;
            }
            continue;
        }

        if char == ' ' && quote_char.is_none() {
            update_split_query(&mut split_query, &temp);
            temp.clear();
        } else {
            temp.push(char);
        }
    }
    update_split_query(&mut split_query, &temp);
    split_query.query = Some(query.to_string());

    split_query
}

#[test]
fn handle_simple() {
    let query = "* search replace";
    assert_eq!(
        split_query(query),
        QuerySplit {
            query: Some(query.to_string()),
            glob: Some("*".to_string()),
            search: Some("search".to_string()),
            replace: Some("replace".to_string()),
        }
    );
}

#[test]
fn handle_double_quotes() {
    let query = "* \"search quotes\" replace";
    assert_eq!(
        split_query(query),
        QuerySplit {
            query: Some(query.to_string()),
            glob: Some("*".to_string()),
            search: Some("search quotes".to_string()),
            replace: Some("replace".to_string()),
        }
    );
}

#[test]
fn handle_single_quotes() {
    let query = "* 'search quotes' replace";
    assert_eq!(
        split_query(query),
        QuerySplit {
            query: Some(query.to_string()),
            glob: Some("*".to_string()),
            search: Some("search quotes".to_string()),
            replace: Some("replace".to_string()),
        }
    );
}

#[test]
fn handle_empty() {
    let query = "";
    assert_eq!(
        split_query(query),
        QuerySplit {
            query: Some(query.to_string()),
            glob: None,
            search: None,
            replace: None,
        }
    );
}

#[test]
fn handle_empty_quotes() {
    let query = "src \"\"";
    assert_eq!(
        split_query(query),
        QuerySplit {
            query: Some(query.to_string()),
            glob: Some("src".to_string()),
            search: None,
            replace: None,
        }
    );
}

#[test]
fn ignore_additional_spaces() {
    let query = "  *  search  replace  ";
    assert_eq!(
        split_query(query),
        QuerySplit {
            query: Some(query.to_string()),
            glob: Some("*".to_string()),
            search: Some("search".to_string()),
            replace: Some("replace".to_string()),
        }
    );
}

#[test]
fn ignore_additional_words() {
    let query = "* search replace extra";
    assert_eq!(
        split_query(query),
        QuerySplit {
            query: Some(query.to_string()),
            glob: Some("*".to_string()),
            search: Some("search".to_string()),
            replace: Some("replace".to_string()),
        }
    );
}

#[test]
fn query_display_with_colors() {
    let query = QuerySplit {
        query: Some("* search replace".to_string()),
        glob: Some("*".to_string()),
        search: Some("search".to_string()),
        replace: Some("replace".to_string()),
    };
    assert_eq!(
        query.display_with_colors(),
        format!(
            "{} {} {}",
            "*".to_string().stylize().bold().blue(),
            "search".to_string().stylize().bold().yellow(),
            "replace".to_string().stylize().bold().green()
        )
    );
}
