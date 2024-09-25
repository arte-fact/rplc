use std::cmp::max;
use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;

use crossterm::style::Stylize;
use tokio::sync::Mutex;

use crate::libs::decorate_file_content::decorate_file_content;
use crate::libs::scrollbar::display_scrollbar;
use crate::libs::syntax_highlight::highlight_line;
use crate::libs::terminal::{print_at, screen_width};

lazy_static! {
    static ref WINDOWS: Arc<Mutex<HashMap<String, Window>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub async fn create_and_store_window(key: String, attrs: Vec<WindowAttr>) -> Result<Window, Error> {
    let mut window = Window::default();
    for attr in attrs {
        window.update_attribute(attr)?;
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
    window.update_attribute(attr);
    Ok(windows.get(key).cloned())
}

pub enum WindowAttr {
    Title(String),
    Width(usize),
    Height(Option<usize>),
    Top(usize),
    Left(usize),
    Content(Vec<String>),
    Footer(String),
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
    top: usize,
    left: usize,
    width: usize,
    height: Option<usize>,
    decorated: bool,
    scrollable: bool,
    scroll_offset: usize,
    code_highlight: Option<String>,
}

impl Window {
    pub fn draw(&self) -> Result<(), Error> {
        let pad = if self.decorated { 2 } else { 0 };
        let height = self.height.unwrap_or(self.content.len());

        let mut content = self
            .content
            .clone()
            .iter()
            .skip(max(0, self.scroll_offset))
            .take(height - pad)
            .map(|s| match self.code_highlight {
                Some(ref lang) => {
                    let line_len = s.len();
                    let width = self.width - pad;
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

        if self.content.len() < height {
            content.extend(vec![
                "".to_string();
                max(height - pad - self.content.len(), 0)
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
                self.content.len() as usize,
                self.top + 1,
                self.height.unwrap_or(self.content.len()) - 2,
                self.left + self.width - 1,
            )?;
        }
        for (i, line) in content.iter().enumerate() {
            print_at(
                self.left as u16,
                (self.top + i) as u16,
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
        if !self.scrollable {
            return Ok(self);
        }
        let next_offset = self.scroll_offset as isize + offset;
        let max_offset = self.content.len() as isize - self.height.unwrap_or(self.content.len()) as isize + 2;
        let offset = if next_offset < 0 {
            0
        } else 
        if next_offset as usize > max_offset as usize {
            max_offset as usize
        } else {
            next_offset as usize
        };
        self.scroll(offset)
    }

    pub fn scrollable(&mut self, scrollable: bool) -> Result<&Self, Error> {
        self.scrollable = scrollable;
        Ok(self)
    }
    pub fn clear(&self) -> Result<&Self, Error> {
        for (i, _) in self.content.iter().enumerate() {
            print_at(
                self.left as u16,
                (self.top + i) as u16,
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

    pub fn update_attribute(&mut self, attr: WindowAttr) -> Result<&Self, Error> {
        match attr {
            WindowAttr::Title(title) => self.title = title,
            WindowAttr::Content(content) => self.content = content,
            WindowAttr::Footer(footer) => self.footer = footer,
            WindowAttr::Decorated(decorated) => self.decorated = decorated,
            WindowAttr::Scrollable(scrollable) => self.scrollable = scrollable,
            WindowAttr::Scroll(scroll) => self.scroll_offset = scroll,
            WindowAttr::Highlight(highlight) => self.code_highlight = highlight,
            WindowAttr::Width(width) => self.width = width,
            WindowAttr::Height(height) => self.height = height,
            WindowAttr::Top(top) => self.top = top,
            WindowAttr::Left(left) => self.left = left,
        }
        Ok(self)
    }
}
