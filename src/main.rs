#[macro_use]
extern crate lazy_static;

pub mod classic;

mod libs;

use std::io::{stdout, Error};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use chrono::Local;
use clap::Parser;
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::style::Stylize;

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
};

use self::classic::classic_mode;
use self::libs::files::{list_glob_files, store_glob_files};
use self::libs::split_query::{split_query, QuerySplit};
use self::libs::state::{get_file, get_files_names, get_key_value, store_key_value};
use self::libs::terminal::{get_screen_size, hide_cursor, print_at, screen_height, screen_width};
use self::libs::ui::window::{create_and_store_window, get_window, store_window, WindowAttr};

static FILE_COUNT: AtomicUsize = AtomicUsize::new(0);
static REPLACED_COUNT: AtomicUsize = AtomicUsize::new(0);
static TOTAL_LINES: AtomicUsize = AtomicUsize::new(0);
static SCROLL_OFFSET: AtomicUsize = AtomicUsize::new(0);

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Opts {
    #[arg(help = "Glob pattern to search for files")]
    glob: Option<String>,

    #[arg(help = "Query to search for")]
    query: Option<String>,

    #[arg(help = "Substitute to replace query with")]
    substitute: Option<String>,

    #[arg(short, long, help = "Write changes to files")]
    write: bool,

    #[arg(short, long, help = "Classic mode")]
    classic: bool,
}


async fn handle_user_query_with_errors(user_query: &String) {
    match handle_user_query(&user_query).await {
        Ok(_) => (),
        Err(e) => debug!("Error: {}", e),
    }
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
        screen_width() as u16 - scroll_help.len() as u16,
        (screen_height() - 1) as u16,
        &scroll_help,
    )
}

async fn handle_search_and_replace(
    search: Option<String>,
    replacement: Option<String>,
) -> Result<(), std::io::Error> {
    REPLACED_COUNT.store(0, Ordering::SeqCst);
    FILE_COUNT.store(0, Ordering::SeqCst);
    TOTAL_LINES.store(0, Ordering::SeqCst);

    let files_names = get_files_names().await;

    for file_name in files_names.clone().iter() {
        //
    }

    let resume = match (search, replacement) {
        (Some(_), Some(_)) => format!(
            " {} changes in {} files: ",
            FILE_COUNT.load(Ordering::SeqCst),
            files_names.len()
        ),
        (Some(_), None) => format!(
            " {} matches in {} files: ",
            FILE_COUNT.load(Ordering::SeqCst),
            files_names.len()
        ),
        _ => format!(" {} files found: ", files_names.len()),
    }
    .black()
    .on_green()
    .bold()
    .to_string();

    // display_files_tree(6, 0, files_names)?;

    print_at(0, 4, &resume)?;

    Ok(())
}

async fn handle_user_query(user_query: &String) -> Result<(), std::io::Error> {
    print_help()?;
    let split = split_query(user_query);
    split.print()?;

    let last_time = get_key_value("time").await.unwrap_or("".to_string());
    let last_query = get_key_value("user_query").await.unwrap_or("".to_string());

    store_key_value("user_query".to_string(), user_query.clone()).await;
    store_key_value("time".to_string(), Local::now().to_string()).await;

    let elapsed = Local::now().timestamp() - last_time.parse::<i64>().unwrap_or(0);

    if elapsed < 300 && user_query == &last_query {
        return Ok(());
    }

    tokio::task::spawn(async move {
        match display_results(split).await {
            Ok(_) => (),
            Err(e) => debug!("Error: {}", e),
        };
    });

    Ok(())
}

async fn display_results(split: QuerySplit) -> Result<(), std::io::Error> {
    let glob = match &split.glob {
        Some(glob) => glob,
        None => return Ok(()),
    };
    store_glob_files(glob).await?;
    let files = list_glob_files(glob)?;
    print_at(0, 3, &format!("{} files found", files.len()))?;
    let mut top = 4;
    for file_path in files.iter() {
        let path = match file_path.to_str() {
            Some(path) => path,
            None => continue,
        };
        let content = match get_file(&path).await {
            Some(content) => content,
            None => "No content.".to_string(),
        };
        code_window(&path, &content, top as usize).await?;
        top += (content.lines().count() + 2) as u16;
        if top as usize > screen_height() {
            break;
        }
    }

    Ok(())
}

async fn code_window(path: &str, content: &str, top: usize) -> Result<(), std::io::Error> {
    let height = screen_height() - top;
    let width = screen_width();

    let highlight = match path.split('.').last() {
        Some(highlight) => Some(highlight.to_string()),
        None => None,
    };

    create_and_store_window(
        "result".to_string(),
        vec![
            WindowAttr::Title(path.to_string()),
            WindowAttr::Content(content.lines().map(|x| x.to_string()).collect()),
            WindowAttr::Footer("Footer".to_string()),
            WindowAttr::Top(top as usize),
            WindowAttr::Left(0),
            WindowAttr::Width(width as usize),
            WindowAttr::Height(Some(height as usize)),
            WindowAttr::Decorated(true),
            WindowAttr::Scrollable(true),
            WindowAttr::Scroll(0),
            WindowAttr::Highlight(highlight),
        ],
    )
    .await?
    .draw()
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

async fn interactive_mode() -> Result<(), std::io::Error> {
    hide_cursor()?;
    execute!(stdout(), EnterAlternateScreen)?;
    execute!(stdout(), Clear(ClearType::All))?;
    get_screen_size()?;
    enable_raw_mode()?;
    let mut user_query = "src/**/*".to_string();
    handle_user_query_with_errors(&user_query).await;
    loop {
        if poll(Duration::from_millis(500))? {
            match read()? {
                Event::Key(event)
                    if [KeyCode::Esc].contains(&event.code)
                        || event.code == KeyCode::Char('c')
                            && event.modifiers == KeyModifiers::CONTROL =>
                {
                    disable_raw_mode()?;
                    execute!(stdout(), crossterm::terminal::LeaveAlternateScreen)?;
                    println!("Exiting...");
                    return Ok(());
                }
                Event::Key(event) if event.code == KeyCode::Down => {
                    scroll_window("result", 5).await?;
                }
                Event::Key(event) if event.code == KeyCode::Up => {
                    scroll_window("result", -5).await?;
                }
                Event::Key(event) => {
                    user_query = handle_key_event(event, &user_query.clone());
                    handle_user_query_with_errors(&user_query).await;
                }
                _ => (),
            }
        } else {
            // Timeout expired and no `Event` is available
        }
    }
}

async fn scroll_window(name: &str, offset: isize) -> Result<(), std::io::Error> {
    let mut window = match get_window(name).await {
        Some(window) => window,
        None => return Ok(()),
    };
    let scroll = window.scroll_by(offset)?;
    let _ = &scroll.draw()?;
    store_window(name.to_string(), window).await
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let opts: Opts = Opts::parse();

    if opts.classic {
        return classic_mode(&opts).await;
    }

    interactive_mode().await
}
