use std::cmp::{max, min};
use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;

use crossterm::style::Stylize;
use tokio::sync::Mutex;

use crate::libs::scrollbar::display_scrollbar;
use crate::libs::syntax_highlight::highlight_line;
use crate::libs::terminal::print_at;

lazy_static! {
    static ref WINDOWS: Arc<Mutex<HashMap<String, Window>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub async fn create_and_store_window(key: String, attrs: Vec<WindowAttr>) -> Result<Window, Error> {
    if let Some(window) = get_window(&key).await {
        window.clear()?;
    }
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

pub async fn get_window(key: &str) -> Option<Window> {
    let windows = WINDOWS.lock().await;
    windows.get(key).cloned()
}

pub enum WindowAttr {
    Title(Option<String>),
    Footer(Option<String>),
    Width(usize),
    Height(Option<usize>),
    Top(usize),
    Left(usize),
    Content(Vec<String>),
    Decorated(bool),
    Scrollable(bool),
    Scroll(usize),
    Highlight(Option<String>),
    DecorationStyle(DecorationStyle),
}

#[derive(Default, Clone)]
pub struct Window {
    pub title: Option<String>,
    pub content: Vec<String>,
    pub footer: Option<String>,
    pub top: usize,
    pub left: usize,
    pub width: usize,
    pub height: Option<usize>,
    pub decorated: bool,
    pub scrollable: bool,
    pub scroll_offset: usize,
    pub code_highlight: Option<String>,
    pub decoration_style: DecorationStyle,
}

impl Window {
    pub fn highlight_content(&self) -> Result<(), Error> {
        let content = self
            .content
            .clone()
            .iter()
            .take(self.height.unwrap_or(self.content.len()) - 2)
            .map(|s| match self.code_highlight {
                Some(ref lang) => {
                    let mut s = s.clone().chars().take(self.width - 2).collect::<String>();
                    s.push_str(" ".repeat(self.width - 2 - s.len()).as_str());
                    return highlight_line(&s, lang).unwrap_or_else(|_| s.clone());
                }
                None => s.clone(),
            })
            .collect::<Vec<String>>();

        for (i, line) in content.iter().enumerate() {
            print_at(
                (self.left + 1) as u16,
                (self.top + i + 1) as u16,
                line.as_str(),
            )?;
        }

        Ok(())
    }

    pub fn visible_content(&self) -> Vec<String> {
        self.content
            .clone()
            .iter()
            .skip(max(0, self.scroll_offset))
            .take(self.height.unwrap_or(self.content.len()) - 2)
            .map(|s| s.clone().chars().take(self.width - 2).collect::<String>())
            .collect()
    }

    pub fn draw(&self) -> Result<(), Error> {
        self.clear()?;
        let height = self.height.unwrap_or(self.content.len());
        let height = height - 2;
        let left = self.left + 1;
        let top = self.top + 1;

        if self.decorated {
            print_window_decoration(self.clone())?;
        }

        for (i, line) in self.visible_content().iter().enumerate() {
            print_at(
                left as u16,
                (top + i) as u16,
                line.as_str().reset().to_string().as_str(),
            )?;
        }
        if self.scrollable {
            display_scrollbar(
                self.scroll_offset,
                self.content.len() as usize,
                top,
                height,
                left + self.width - 1,
            )?;
        }

        if self.code_highlight.is_some() {
            self.highlight_content()?;
        }

        Ok(())
    }

    pub fn scroll(&mut self, offset: usize) -> Result<&Self, Error> {
        self.scroll_offset = offset;
        self.draw()?;
        Ok(self)
    }

    pub fn scroll_by(&mut self, offset: isize) -> Result<&Self, Error> {
        if !self.scrollable {
            return Ok(self);
        }
        let next_offset = self.scroll_offset as isize + offset;
        let max_offset = self.content.len() as isize - self.height.unwrap_or(self.content.len()) as isize + 2;

        self.scroll(max(0, min(next_offset as usize, max_offset as usize)))
    }

    pub fn clear(&self) -> Result<&Self, Error> {
        for i in 0..self.height.unwrap_or(self.content.len()) {
            print_at(
                self.left as u16,
                (self.top + i) as u16,
                &" ".repeat(self.width),
            )?;
        }
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
            WindowAttr::DecorationStyle(decoration_style) => {
                self.decoration_style = decoration_style
            }
        }
        Ok(self)
    }
}

fn print_window_decoration(window: Window) -> Result<(), Error> {
    let top_left = decoration_style_charset(window.decoration_style)
        .chars()
        .nth(0)
        .unwrap()
        .to_string();
    let top_right = decoration_style_charset(window.decoration_style)
        .chars()
        .nth(1)
        .unwrap()
        .to_string();
    let bottom_left = decoration_style_charset(window.decoration_style)
        .chars()
        .nth(2)
        .unwrap()
        .to_string();
    let bottom_right = decoration_style_charset(window.decoration_style)
        .chars()
        .nth(3)
        .unwrap()
        .to_string();
    let top_line = decoration_style_charset(window.decoration_style)
        .chars()
        .nth(4)
        .unwrap()
        .to_string();
    let bottom_line = decoration_style_charset(window.decoration_style)
        .chars()
        .nth(5)
        .unwrap()
        .to_string();
    let left_line = decoration_style_charset(window.decoration_style)
        .chars()
        .nth(6)
        .unwrap()
        .to_string();
    let right_line = decoration_style_charset(window.decoration_style)
        .chars()
        .nth(7)
        .unwrap()
        .to_string();

    let height = window.height.unwrap_or(window.content.len());

    print_at(
        window.left as u16,
        (window.top) as u16,
        &format!(
            "{}{}{}",
            top_left,
            top_line.repeat(window.width - 2),
            top_right
        )
        .dark_grey()
        .to_string(),
    )?;
    for i in 1..(window.height.unwrap_or(window.content.len()) - 1) {
        print_at(
            window.left as u16,
            (window.top + i) as u16,
            &format!(
                "{}{}{}",
                left_line,
                " ".repeat(window.width - 2),
                right_line
            )
            .dark_grey()
            .to_string(),
        )?;
    }
    print_at(
        window.left as u16,
        (window.top + height - 1) as u16,
        &format!(
            "{}{}{}",
            bottom_left,
            bottom_line.repeat(window.width - 2),
            bottom_right
        )
        .dark_grey()
        .to_string(),
    )?;

    if let Some(title) = window.title {
        print_at(
            (window.left + 1) as u16,
            window.top as u16,
            &format!(
                " {} ",
                title
                    .chars()
                    .take(window.width - 2)
                    .collect::<String>()
                    .grey()
                    .to_string()
            ),
        )?;
    }
    if let Some(footer) = window.footer {
        print_at(
            (window.left + window.width - footer.len() - 3) as u16,
            (window.top + height + 1) as u16,
            &format!(" {} ", footer.as_str())
                .chars()
                .take(window.width - 2)
                .collect::<String>()
                .grey()
                .to_string(),
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum DecorationStyle {
    Rounded,
    Single,
    Double,
    Shadow,
    None,
}

impl Default for DecorationStyle {
    fn default() -> Self {
        DecorationStyle::Shadow
    }
}

pub fn decoration_style_charset(style: DecorationStyle) -> String {
    match style {
        DecorationStyle::Rounded => "╭╮╰╯──││",
        DecorationStyle::Single => "┌┐└┘──││",
        DecorationStyle::Double => "╔╗╚╝══║║",
        DecorationStyle::Shadow => "┌┐└┘──││",
        DecorationStyle::None => "        ",
    }
    .to_string()
}
