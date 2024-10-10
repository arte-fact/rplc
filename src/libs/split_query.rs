use std::io::Error;

use crossterm::style::Stylize;

use super::state::store_key_value;
use super::terminal::{clear_lines, print_at};

#[derive(Debug, PartialEq, Default, Clone)]
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
        print_at(self.len() as u16, 2, "█")

    }

    pub async fn store(&self) {
        if let Some(query) = &self.query {
            store_key_value("query".to_string(), query.to_string()).await;
        }
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

pub async fn split_query(query: &str) -> QuerySplit {
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

    split_query.store().await;

    split_query
}
