use std::cmp::max;
use std::io::Error;
use std::time::Duration;

use chrono::Local;
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::style::Stylize;

use crate::libs::file_tree::Tree;
use crate::libs::files::store_glob_files;
use crate::libs::split_query::split_query;
use crate::libs::state::{get_key_value, store_key_value};
use crate::libs::store::query::store_query;
use crate::libs::store::tree::{store_selected, store_tree};
use crate::libs::store::windows::{get_window, store_window};
use crate::libs::terminal::{
    clear_screen, disable_raw_mode, enable_raw_mode, enter_alternate_screen, get_screen_size, hide_cursor, leave_alternate_screen, print_at, screen_height, screen_width, set_screen_size
};
use crate::libs::windows::code::code_window;
use crate::libs::windows::file_tree::{file_tree_window, select_file_by};

fn init() -> Result<(), std::io::Error> {
    hide_cursor()?;
    enter_alternate_screen()?;
    clear_screen()?;
    enable_raw_mode()?;
    get_screen_size()?;

    Ok(())
}

pub async fn interactive_mode() -> Result<(), std::io::Error> {
    crate::log!("Starting interactive mode");
    init()?;

    let mut user_query = "**/*".to_string();

    handle_user_query(&user_query).await?;
    redraw().await?;
    loop {
        if poll(Duration::from_millis(500))? {
            match read()? {
                Event::Key(event)
                    if [KeyCode::Esc].contains(&event.code)
                        || event.code == KeyCode::Char('c')
                            && event.modifiers == KeyModifiers::CONTROL =>
                {
                    leave_alternate_screen()?;
                    disable_raw_mode()?;
                    println!("Exiting...");
                    return Ok(());
                }
                Event::Key(event)
                    if event.code == KeyCode::Down && event.modifiers == KeyModifiers::CONTROL =>
                {
                    scroll_window("result", 5).await?;
                }
                Event::Key(event)
                    if event.code == KeyCode::Up && event.modifiers == KeyModifiers::CONTROL =>
                {
                    scroll_window("result", -5).await?;
                }
                Event::Key(event) if event.code == KeyCode::Up => {
                    select_file_by(-1).await?;
                    redraw().await?;
                }
                Event::Key(event) if event.code == KeyCode::Down => {
                    select_file_by(1).await?;
                    redraw().await?;
                }
                Event::Key(event) => {
                    user_query = handle_key_event(event, &user_query.clone());
                    debounce_user_query(&user_query).await?;
                }
                Event::Resize(w, h) => {
                    set_screen_size(w as usize, h as usize);
                    redraw().await?;
                }
                _ => (),
            }
        } else {
            // Timeout expired and no `Event` is available
        }
    }
}

fn handle_key_event(event: crossterm::event::KeyEvent, user_query: &String) -> String {
    let mut user_query = user_query.clone();
    match event.kind {
        KeyEventKind::Press => {
            let input = event.code;
            match input {
                KeyCode::Char(c) => {
                    user_query.push(c);
                }
                KeyCode::Backspace => {
                    user_query.pop();
                }
                _ => (),
            }
        }
        _ => (),
    }
    user_query
}

async fn debounce_user_query(user_query: &String) -> Result<(), std::io::Error> {
    print_help()?;
    let split = split_query(user_query).await;
    split.print()?;

    let last_time = get_key_value("time").await.unwrap_or("".to_string());
    let last_query = get_key_value("user_query").await.unwrap_or("".to_string());

    store_key_value("user_query".to_string(), user_query.clone()).await;
    store_key_value("time".to_string(), Local::now().to_string()).await;

    let elapsed = Local::now().timestamp() - last_time.parse::<i64>().unwrap_or(0);

    if elapsed < 60 && user_query == &last_query {
        return Ok(());
    }

    let user_query = user_query.clone();
    tokio::spawn(async move {
        match handle_user_query(&user_query).await {
            Ok(_) => (),
            Err(e) => {
                leave_alternate_screen()
                    .unwrap_or_else(|e| eprintln!("{}", e));
                eprintln!("{}", e)
            }
        }
    });
    Ok(())
}

async fn handle_user_query(user_query: &String) -> Result<(), std::io::Error> {
    let user_query = user_query.clone();
    tokio::spawn(async move {
        handle_user_query_task(&user_query).await.unwrap_or_else(|e| print_at(0, 0, &e.to_string()).unwrap());
    });

    Ok(())
}

async fn handle_user_query_task(user_query: &String) -> Result<(), std::io::Error> {
    let split = split_query(user_query).await;
    split.print()?;
    store_query(&split).await;
    let paths = store_glob_files(&split).await?;
    if paths.is_empty() {
        store_selected(None).await;
        return Ok(());
    }
    
    let selected = paths.iter().filter(|path| path.is_file()).next().cloned();
    let tree = Tree::from_path_vec(&paths);
    store_tree(&tree).await;
    store_selected(selected).await;
    file_tree_window().await?;
    code_window().await?;
    Ok(())
}

async fn scroll_window(name: &str, offset: isize) -> Result<(), std::io::Error> {
    let mut window = match get_window(name).await {
        Some(window) => window,
        None => return Ok(()),
    };
    let scroll = window.scroll_by(offset)?;
    let _ = &scroll.draw()?;
    Ok(())
}

async fn redraw() -> Result<(), std::io::Error> {
    print_help()?;
    file_tree_window().await?;
    code_window().await?;
    Ok(())
}

fn print_help() -> Result<(), Error> {
    let help = format!(
        "Query format: {} {} {}",
        "<glob>".stylize().blue().bold(),
        "<query>".stylize().yellow().bold(),
        "<replacement>".stylize().green().bold()
    );
    print_at(0, 0, &help)?;
    let scroll_help = format!("↑/↓ to scroll, ESC to exit, CTRL+C to exit, ENTER to write changes");
    print_at(
        max(screen_width() - scroll_help.len(), 0) as u16,
        (screen_height() - 1) as u16,
        &scroll_help,
    )
}
