mod libs;

use std::io::stdout;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use clap::Parser;
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::style::{Color, Stylize};

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
};
use glob::glob;
use tokio::fs::read_to_string;

static FILE_COUNT: AtomicUsize = AtomicUsize::new(0);
static REPLACED_COUNT: AtomicUsize = AtomicUsize::new(0);

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
    query: &str,
    substitute: &str,
    path: &str,
) -> Result<Vec<String>, std::io::Error> {
    let content = match read_to_string(path).await {
        Ok(content) => content,
        Err(e) => {
            return Ok(vec![format!("Could not read file {}: {}", path, e)]);
        }
    };

    let mut code: Vec<String> = Vec::new();
    if !content.contains(query) {
        return Ok(code);
    }
    code.push(format!(
        "╭{}\n",
        format!(" {} ", path).stylize().with(Color::Grey)
    ));
    FILE_COUNT.fetch_add(1, Ordering::SeqCst);

    let lines = content.lines();
    let mut line_number = 1;
    let mut skipped = false;

    for line in lines {
        if line.contains(query) {
            skipped = false;
            let line = line.replace(
                query,
                &format!("{}", substitute.stylize().with(Color::Green).bold()),
            );

            let changes = line.matches(substitute).count();
            REPLACED_COUNT.fetch_add(changes, Ordering::SeqCst);

            code.push(format!(
                "│ {:<4} {:<80}\n",
                line_number.to_string().stylize().with(Color::DarkGrey),
                line
            ));
        } else {
            if !skipped {
                code.push(format!(
                    "│ {} \n",
                    "...".to_string().stylize().with(Color::DarkGrey)
                ));
                skipped = true;
            }
        }
        line_number += 1;
    }
    code.push("╰────────".to_string());

    Ok(code)
}

async fn replace_in_file(query: &str, substitute: &str, path: &str) -> Result<(), std::io::Error> {
    let content = match read_to_string(path).await {
        Ok(content) => content,
        Err(_e) => {
            return Ok(());
        }
    };
    if !content.contains(query) {
        return Ok(());
    }
    let new_content = content.replace(query, substitute);

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
            let files: Vec<PathBuf> = files.take(100).filter_map(|x| x.ok()).collect();
            Ok(files)
        }
    }
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
    let query = opts.query.clone().unwrap();
    let substitute = opts.substitute.clone().unwrap();
    let glob = opts.glob.clone().unwrap();

    println!(
        "rplc {} with {} in {}:\n",
        &query
            .clone()
            .stylize()
            .with(crossterm::style::Color::Yellow)
            .bold(),
        &substitute
            .clone()
            .stylize()
            .with(crossterm::style::Color::Green)
            .bold(),
        &glob.clone().stylize().with(crossterm::style::Color::Green)
    );

    let files = list_glob_files(&glob)?;
    for file in &files {
        if !file.is_file() {
            continue;
        }
        for line in display_changes_in_file(&query, &substitute, file.to_str().unwrap()).await? {
            println!("{}", line);
        }
    }

    if opts.write || prompt_user() {
        for file in &files {
            replace_in_file(&query, &substitute, file.to_str().unwrap()).await?;
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

fn print_at(x: u16, y: u16, text: &str) {
    execute!(
        stdout(),
        crossterm::cursor::MoveTo(x, y),
        crossterm::style::Print(text)
    )
    .unwrap();
}

async fn interactive_mode() -> Result<(), std::io::Error> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut user_query = String::new();
    handle_user_query(&user_query).await?;
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
                Event::Key(event) => {
                    user_query = handle_key_event(event, &user_query.clone());
                    handle_user_query(&user_query).await?;
                }
                _ => (),
            }
        } else {
            // Timeout expired and no `Event` is available
        }
    }
}

fn print_help() {
    let help = format!(
        "Query format: {} {} {}",
        "<glob>".stylize().blue().bold(),
        "<query>".stylize().yellow().bold(),
        "<replacement>".stylize().green().bold()
    );
    print_at(0, 0, &help);
}

async fn handle_search_and_replace(
    glob_search: &str,
    query: &str,
    replacement: &str,
) -> Result<(), std::io::Error> {
    REPLACED_COUNT.store(0, Ordering::SeqCst);
    FILE_COUNT.store(0, Ordering::SeqCst);
    let (_, height) = crossterm::terminal::size()?;

    execute!(stdout(), Clear(ClearType::Purge))?;
    execute!(stdout(), Clear(ClearType::All))?;
    print_help();

    cursor_at(0, 2);
    execute!(stdout(), Clear(ClearType::CurrentLine))?;
    let mut i = 0;
    for file in list_glob_files(glob_search)?.iter() {
        if !file.is_file() {
            continue;
        }
        for line in display_changes_in_file(query, replacement, file.to_str().unwrap()).await? {
            i += 1;
            if i >= height as usize - 6 {
                break;
            }
            print_at(0, (i + 5) as u16, &line);
        }
    }
    print_at(
        0,
        2,
        glob_search.stylize().blue().bold().to_string().as_str(),
    );
    print_at(
        (glob_search.len() + 1) as u16,
        2,
        query.stylize().yellow().bold().to_string().as_str(),
    );
    print_at(
        0,
        4,
        &format!(
            "{} matches in {} files:",
            REPLACED_COUNT
                .load(std::sync::atomic::Ordering::SeqCst)
                .to_string()
                .stylize()
                .green()
                .bold(),
            FILE_COUNT
                .load(std::sync::atomic::Ordering::SeqCst)
                .to_string()
                .stylize()
                .green()
                .bold(),
        ),
    );
    cursor_at(glob_search.len() as u16 + 1 + query.len() as u16, 2);
    if query == replacement {
        return Ok(());
    }
    print_at(
        (glob_search.len() + 1 + query.len() + 1) as u16,
        2,
        replacement.stylize().green().bold().to_string().as_str(),
    );
    cursor_at(
        glob_search.len() as u16 + 1 + query.len() as u16 + 1 + replacement.len() as u16,
        2,
    );
    Ok(())
}

async fn handle_user_query(user_query: &String) -> Result<(), std::io::Error> {
    execute!(stdout(), Clear(ClearType::Purge))?;
    execute!(stdout(), Clear(ClearType::All))?;
    let split_query: Vec<&str> = user_query.split(" ").collect();
    REPLACED_COUNT.store(0, Ordering::SeqCst);
    FILE_COUNT.store(0, Ordering::SeqCst);

    match split_query.len() {
        1 => handle_glob_search(user_query).await?,
        2 => {
            if split_query[1].len() > 0 {
                handle_search_and_replace(split_query[0], split_query[1], split_query[1]).await?
            } else {
                handle_glob_search(user_query).await?
            }
        }
        3 => handle_search_and_replace(split_query[0], split_query[1], split_query[2]).await?,
        _ => handle_search_and_replace(split_query[0], split_query[1], split_query[2]).await?,
    }

    Ok(())
}

fn cursor_at(x: u16, y: u16) {
    execute!(stdout(), crossterm::cursor::MoveTo(x, y)).unwrap();
}

async fn handle_glob_search(user_query: &String) -> Result<(), std::io::Error> {
    let (_, height) = crossterm::terminal::size()?;
    execute!(stdout(), Clear(ClearType::Purge))?;
    execute!(stdout(), Clear(ClearType::All))?;

    print_help();

    if user_query.is_empty() {
        cursor_at(0, 2);
        return Ok(());
    }
    let files = list_glob_files(&user_query.trim())?;

    let list_title = format!(
        "Found {}{} files: ",
        files.len(),
        if files.len() >= 100 { "+" } else { "" }
    );
    cursor_at(0, 2);
    execute!(stdout(), Clear(ClearType::CurrentLine))?;

    print_at(0, 4, &list_title);
    for (i, file) in files.iter().enumerate() {
        print_at(0, (i + 6) as u16, file.to_str().unwrap());
        if i >= height as usize - 5 {
            break;
        }
    }

    print_at(
        0,
        2,
        user_query
            .to_string()
            .stylize()
            .blue()
            .bold()
            .to_string()
            .as_str(),
    );
    cursor_at(user_query.len() as u16, 2);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let opts: Opts = Opts::parse();

    if opts.classic {
        return classic_mode(&opts).await;
    }

    interactive_mode().await?;

    Ok(())
}
