#[macro_use]
extern crate lazy_static;

mod libs;

use std::cmp::max;
use std::io::{stdout, Error};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use chrono::Local;
use clap::Parser;
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::style::{Color, Stylize};

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
};
use glob::glob;
use tokio::fs::read_to_string;

use libs::decorate_file_content::{decorate_file_content, happend_changes_in_file};

use self::libs::split_query::{split_query, QuerySplit};
use self::libs::state::{
    clear_files, get_file, get_files_names, get_key_value, store_file, store_key_value,
};
use self::libs::syntax_highlight::highlight_line;
use self::libs::terminal::{
    clear_results, get_screen_size, hide_cursor, print_at, screen_height, screen_width,
};
use self::libs::ui::window::{create_and_store_window, get_window, WindowAttr};

static SCROLL_OFFSET: AtomicUsize = AtomicUsize::new(0);
static FILE_COUNT: AtomicUsize = AtomicUsize::new(0);
static REPLACED_COUNT: AtomicUsize = AtomicUsize::new(0);
static TOTAL_LINES: AtomicUsize = AtomicUsize::new(0);

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

async fn display_changes_in_file(
    search: Option<String>,
    substitute: Option<String>,
    path: &str,
) -> Result<(Vec<String>, usize), std::io::Error> {
    let content = get_file(path).await.unwrap_or("No content.".to_string());
    let extension = path.split('.').last().unwrap_or("txt");
    let content = content
        .lines()
        .map(|x| highlight_line(x, extension).unwrap_or(x.to_string()))
        .collect();

    let query = match search {
        Some(query) => query,
        None => return Ok((content, 0)),
    };

    let substitute = match substitute {
        Some(substitute) => substitute,
        None => query.clone(),
    };

    let (result, changes): _ = happend_changes_in_file(content, &query, &substitute);

    REPLACED_COUNT.fetch_add(changes, Ordering::SeqCst);
    if changes != 0 {
        FILE_COUNT.fetch_add(1, Ordering::SeqCst);
    }
    Ok((result, changes))
}

async fn replace_in_file(
    query: String,
    substitute: String,
    path: &str,
) -> Result<(), std::io::Error> {
    let content = match read_to_string(path).await {
        Ok(content) => content,
        Err(_e) => {
            return Ok(());
        }
    };
    if !content.contains(&query) {
        return Ok(());
    }
    let new_content = content.replace(&query, &substitute);

    std::fs::write(path, new_content)?;
    Ok(())
}

fn list_glob_files(glob_pattern: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    match glob(glob_pattern) {
        Err(e) => {
            println!("Could not list files: {}", e);
            Ok(vec![])
        }
        Ok(files) => {
            let files: Vec<PathBuf> = files.filter_map(|x| x.ok()).collect();
            Ok(files)
        }
    }
}

async fn store_glob_files(glob_pattern: &str) -> Result<(), std::io::Error> {
    clear_files().await;
    match glob(glob_pattern) {
        Err(e) => {
            println!("Could not list files: {}", e);
            return Ok(());
        }
        Ok(files) => {
            let files: Vec<PathBuf> = files.filter_map(|x| x.ok()).collect();
            for file in files.iter() {
                if !file.is_file() {
                    continue;
                }
                let file_name = file.to_str().unwrap_or("Could not read file name");
                let content = match read_to_string(file).await {
                    Ok(content) => content,
                    Err(e) => file_name.to_string() + &format!(": {}", e),
                };
                store_file(file_name.to_string(), content).await;
            }
        }
    }
    Ok(())
}

fn prompt_user() -> bool {
    println!(
        "\nFound {} replacements in {} files.",
        REPLACED_COUNT
            .load(std::sync::atomic::Ordering::SeqCst)
            .to_string()
            .stylize()
            .with(Color::Green),
        FILE_COUNT
            .load(std::sync::atomic::Ordering::SeqCst)
            .to_string()
            .stylize()
            .with(Color::Yellow),
    );
    loop {
        println!("\nDo you want to continue? [y/n]");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "y" => return true,
            "n" => return false,
            _ => println!("Invalid input. Please enter 'y' or 'n'"),
        }
    }
}

async fn classic_mode(opts: &Opts) -> Result<(), std::io::Error> {
    if opts.query.is_none() || opts.substitute.is_none() || opts.glob.is_none() {
        println!("Invalid input. Please enter <GLOB> <QUERY> <SUBSTITUTE>");
        return Ok(());
    }
    let query = &opts.query;
    let substitute = &opts.substitute;
    let glob = opts.glob.clone();

    println!(
        "rplc {} with {} in {}:\n",
        query
            .clone()
            .unwrap_or("".to_string())
            .stylize()
            .with(crossterm::style::Color::Yellow)
            .bold(),
        substitute
            .clone()
            .unwrap_or("".to_string())
            .stylize()
            .with(crossterm::style::Color::Green)
            .bold(),
        &glob
            .clone()
            .unwrap_or("".to_string())
            .stylize()
            .with(crossterm::style::Color::Green)
    );

    let glob = glob.unwrap_or("".to_string());

    let files = list_glob_files(&glob)?;
    for file in &files {
        if !file.is_file() {
            continue;
        }
        let (lines, _) =
            display_changes_in_file(query.clone(), substitute.clone(), file.to_str().unwrap())
                .await?;
        for line in &lines {
            println!("{}", line);
        }
    }

    if opts.write || prompt_user() {
        for file in &files {
            replace_in_file(
                query.clone().unwrap_or("".to_string()),
                substitute.clone().unwrap_or("".to_string()),
                file.to_str().unwrap(),
            )
            .await?;
        }
        println!("{:?} replacements were made.", REPLACED_COUNT);
        return Ok(());
    }
    println!("No changes were made.");
    Ok(())
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
                    match get_window("result").await {
                        Some(window) => window.clone().scroll_by(1)?.draw()?,
                        None => return Ok(()),
                    }
                }
                Event::Key(event) if event.code == KeyCode::Up => {
                    match get_window("result").await {
                        Some(window) => window.clone().scroll_by(-1)?.draw()?,
                        None => return Ok(()),
                    }
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
    let scroll_offset = SCROLL_OFFSET.load(Ordering::SeqCst);
    let height = screen_height();
    clear_results()?;

    let mut i = 0;
    TOTAL_LINES.store(0, Ordering::SeqCst);

    let files_names = get_files_names().await;

    for file_name in files_names.clone().iter() {
        fun_name(
            &search,
            &replacement,
            &file_name,
            &mut i,
            scroll_offset,
            height,
        )
        .await?;
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

async fn fun_name(
    search: &Option<String>,
    replacement: &Option<String>,
    file_name: &String,
    i: &mut usize,
    scroll_offset: usize,
    height: usize,
) -> Result<(), Error> {
    let (result, changes) =
        display_changes_in_file(search.clone(), replacement.clone(), file_name).await?;
    if changes == 0 && search.is_some() {
        return Ok(());
    }
    let decorated =
        decorate_file_content(file_name, result.clone(), &format!("{} matches", changes));
    for line in &decorated {
        *i += 1;
        if *i >= scroll_offset + height - 6 {
            break;
        }
        if *i < scroll_offset {
            continue;
        }
        print_at(
            0,
            (max(*i + 5 - scroll_offset, 6)) as u16,
            &format!("{: <80}", &line).to_string(),
        )?;
    }
    *i += 1;
    TOTAL_LINES.fetch_add(result.len(), Ordering::SeqCst);
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
    // handle_search_and_replace(split.search.clone(), split.replace.clone()).await
    
    let first = "src/main.rs";
    let content = match get_file(&first).await {
        Some(content) => content,
        None => return Ok(()),
    };

    let top = 4;
    let height = screen_height() - top;
    let width = screen_width();

    let highlight = match first.split('.').last() {
        Some(highlight) => Some(highlight.to_string()),
        None => None,
    };

    create_and_store_window("result".to_string(), vec![
        WindowAttr::Title(first.to_string()),
        WindowAttr::Content(content.lines().map(|x| x.to_string()).collect()),
        WindowAttr::Footer("Footer".to_string()),
        WindowAttr::Position((0, top)),
        WindowAttr::Size((width - 1, height - 1)),
        WindowAttr::Decorated(true),
        WindowAttr::Scrollable(true),
        WindowAttr::Scroll(0),
        WindowAttr::Highlight(highlight),
    ]).await?.draw()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let opts: Opts = Opts::parse();

    if opts.classic {
        return classic_mode(&opts).await;
    }

    interactive_mode().await
}
