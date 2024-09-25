use std::cmp::{max, min};
use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;

use crossterm::style::Stylize;
use tokio::sync::Mutex;

use crate::debug;
use crate::libs::decorate_file_content::decorate_file_content;
use crate::libs::scrollbar::display_scrollbar;
use crate::libs::syntax_highlight::highlight_line;
use crate::libs::terminal::print_at;

lazy_static! {
    static ref WINDOWS: Arc<Mutex<HashMap<String, Window>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub async fn create_and_store_window(key: String, attrs: Vec<WindowAttr>) -> Result<Window, Error> {
    let mut window = Window::default();
    for attr in attrs {
        match attr {
            WindowAttr::Title(title) => window.title = title,
            WindowAttr::Content(content) => window.content = content,
            WindowAttr::Footer(footer) => window.footer = footer,
            WindowAttr::Position(position) => window.position = position,
            WindowAttr::Size(size) => window.size = size,
            WindowAttr::Decorated(decorated) => window.decorated = decorated,
            WindowAttr::Scrollable(scrollable) => window.scrollable = scrollable,
            WindowAttr::Scroll(scroll) => window.scroll_offset = scroll,
            WindowAttr::Highlight(highlight) => window.code_highlight = highlight,
        }
    }
    store_window(key, window.clone()).await?;
    Ok(window)
}

pub async fn store_window(key: String, window: Window) -> Result<(), Error> {
    let mut windows = WINDOWS.lock().await;
    windows.insert(key, window);
    Ok(())
}

pub async fn update_window(key: String, window: Window) -> Result<(), Error> {
    let mut windows = WINDOWS.lock().await;
    windows.insert(key, window);
    Ok(())
}

pub async fn get_window(key: &str) -> Option<Window> {
    let windows = WINDOWS.lock().await;
    windows.get(key).cloned()
}

pub async fn update_window_attribute<F>(
    key: &str,
    attr: WindowAttr,
) -> Result<Option<Window>, Error> {
    let mut windows = WINDOWS.lock().await;
    let window = match windows.get_mut(key) {
        Some(window) => window,
        None => return Ok(None),
    };

    match attr {
        WindowAttr::Title(title) => window.title = title,
        WindowAttr::Content(content) => window.content = content,
        WindowAttr::Footer(footer) => window.footer = footer,
        WindowAttr::Position(position) => window.position = position,
        WindowAttr::Size(size) => window.size = size,
        WindowAttr::Decorated(decorated) => window.decorated = decorated,
        WindowAttr::Scrollable(scrollable) => window.scrollable = scrollable,
        WindowAttr::Scroll(scroll) => window.scroll_offset = scroll,
        WindowAttr::Highlight(highlight) => window.code_highlight = highlight,
    }
    Ok(windows.get(key).cloned())
}

pub enum WindowAttr {
    Title(String),
    Content(Vec<String>),
    Footer(String),
    Position((usize, usize)),
    Size((usize, usize)),
    Decorated(bool),
    Scrollable(bool),
    Scroll(usize),
    Highlight(Option<String>),
}

#[derive(Default, Clone)]
pub struct Window {
    title: String,
    content: Vec<String>,
    footer: String,
    position: (usize, usize),
    size: (usize, usize),
    decorated: bool,
    scrollable: bool,
    scroll_offset: usize,
    code_highlight: Option<String>,
}

impl Window {
    pub fn draw(&self) -> Result<(), Error> {
        let pad = if self.decorated { 2 } else { 0 };
        let mut content = self
            .content
            .clone()
            .iter()
            .skip(max(0, self.scroll_offset))
            .take(self.size.1 - pad)
            .map(|s| match self.code_highlight {
                Some(ref lang) => {
                    let line_len = s.len();
                    let width = self.size.0;
                    let s = if line_len < width {
                        let padding = " ".repeat(width - line_len - pad);
                        s.clone() + &padding
                    } else {
                        s.clone()
                    };

                    highlight_line(&s, lang).unwrap_or_else(|_| s.clone())
                }
                None => s.clone(),
            })
            .collect::<Vec<String>>();

        if self.content.len() < self.size.1 {
            content.extend(vec![
                "".to_string();
                max(self.size.1 - pad - self.content.len(), 0)
            ]);
        }

        let content = if self.decorated {
            decorate_file_content(&self.title, content, &self.footer)
        } else {
            self.content.clone()
        };
        if self.scrollable {
            display_scrollbar(
                self.scroll_offset,
                self.size.1 as usize,
                self.position.1 + 1,
                self.size.1,
                self.size.0,
            )?;
        }
        for (i, line) in content.iter().enumerate() {
            print_at(
                self.position.0 as u16,
                (self.position.1 + i) as u16,
                line.as_str().reset().to_string().as_str(),
            )?;
        }
        Ok(())
    }
    pub fn scroll(&mut self, offset: usize) -> Result<&Self, Error> {
        self.scroll_offset = offset;
        Ok(self)
    }
    pub fn scroll_by(&mut self, offset: isize) -> Result<&Self, Error> {
        let offset = max(0, self.scroll_offset + offset as usize);
        self.scroll(offset)
    }
    pub fn scrollable(&mut self, scrollable: bool) -> Result<&Self, Error> {
        self.scrollable = scrollable;
        Ok(self)
    }
    pub fn clear(&self) -> Result<&Self, Error> {
        for (i, _) in self.content.iter().enumerate() {
            print_at(
                self.position.0 as u16,
                (self.position.1 + i) as u16,
                &" ".repeat(80),
            )?;
        }
        Ok(self)
    }
    pub fn content(&mut self, content: Vec<String>) -> Result<&Self, Error> {
        self.content = content;
        Ok(self)
    }
    pub fn footer(&mut self, footer: &str) -> Result<&Self, Error> {
        self.footer = footer.to_string();
        Ok(self)
    }
    pub fn title(&mut self, title: &str) -> Result<&Self, Error> {
        self.title = title.to_string();
        Ok(self)
    }
    pub fn size(&mut self, size: (usize, usize)) -> Result<&Self, Error> {
        self.size = size;
        Ok(self)
    }
}
