use std::sync::atomic::AtomicUsize;

use crossterm::style::Stylize;

use crate::libs::file::{display_changes_in_file, replace_in_file};
use crate::libs::files::list_glob_files;
use crate::Opts;

static FILE_COUNT: AtomicUsize = AtomicUsize::new(0);
static REPLACED_COUNT: AtomicUsize = AtomicUsize::new(0);
static TOTAL_LINES: AtomicUsize = AtomicUsize::new(0);
static SCROLL_OFFSET: AtomicUsize = AtomicUsize::new(0);

pub(crate) async fn classic_mode(opts: &Opts) -> Result<(), std::io::Error> {
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

    let files = list_glob_files(&glob).await?;
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

fn prompt_user() -> bool {
    println!(
        "\nFound {} replacements in {} files.",
        REPLACED_COUNT
            .load(std::sync::atomic::Ordering::SeqCst)
            .to_string()
            .green(),
        FILE_COUNT
            .load(std::sync::atomic::Ordering::SeqCst)
            .to_string()
            .yellow()
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
