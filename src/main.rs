use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use clap::Parser;
use crossterm::style::{Color, Stylize};


static FILE_COUNT: AtomicUsize = AtomicUsize::new(0);
static REPLACED_COUNT: AtomicUsize = AtomicUsize::new(0);


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Opts {
    #[arg()]
    query: String,
    substitute: String,
    path: String,

    #[arg(short, long, help = "Replace in all files in the folder")]
    recursive: bool,
    #[arg(short, long, help = "Write changes to files")]
    write: bool,
}

fn display_changes_in_file(query: &str, substitute: &str, path: &str) -> Result<(), std::io::Error>{
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_e) => {
            return Ok(());
        }
    };

    let mut code = String::new();
    if !content.contains(query) {
        return Ok(());
    }
    code.push_str(&format!("╭{}\n", format!(" {} ", path).stylize().with(Color::Grey)));
    FILE_COUNT.fetch_add(1, Ordering::SeqCst);

    let lines = content.lines();
    let mut line_number = 1;
    let mut skipped = false;

    for line in lines {
        if line.contains(query){
            skipped = false;
            let line = line.replace(query, &format!("{}", substitute
                .stylize()
                .with(Color::Green)
                .bold()
            ));

            let changes = line.matches(substitute).count();
            REPLACED_COUNT.fetch_add(changes, Ordering::SeqCst);

            code.push_str(&format!("│ {:<6} {:<80}\n", line_number.to_string().stylize().with(Color::DarkGrey), line));
        } else {
            if !skipped {
                code.push_str(format!("│ {} \n", "...".to_string().stylize().with(Color::DarkGrey)).as_str());
                skipped = true;
            }
        }
        line_number += 1;
    }
    code.push_str("╰\n");
    println!("{}", code);



    Ok(())
}

fn replace_in_file(query: &str, substitute: &str, path: &str) -> Result<(), std::io::Error>{
    let content = match  std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_e) => {
            return Ok(());
        }
    };
    if !content.contains(query){
        return Ok(());
    }
    let new_content = content.replace(query, substitute);

    std::fs::write(path, new_content)?;
    Ok(())
}

fn list_files_in_folder(folder: &str) -> Result<Vec<PathBuf>, std::io::Error>{
    let mut files = Vec::new();
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.append(&mut list_files_in_folder(path.to_str().unwrap())?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}

fn prompt_user() -> bool {
    println!("\nFound {} replacements in {} files.", 
        REPLACED_COUNT.load(std::sync::atomic::Ordering::SeqCst).to_string().stylize().with(Color::Green),
        FILE_COUNT.load(std::sync::atomic::Ordering::SeqCst).to_string().stylize().with(Color::Yellow),
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


fn main() -> Result<(), std::io::Error>{
    let opts: Opts = Opts::parse();
    println!("rplc {} with {} in folder {}:\n", 
        opts.query.clone().stylize().with(crossterm::style::Color::Yellow).bold(),
        opts.substitute.clone().stylize().with(crossterm::style::Color::Green).bold(), 
        opts.path.clone().stylize().with(crossterm::style::Color::Green)
    );

    if !opts.write {
        if !opts.recursive {
            display_changes_in_file(&opts.query, &opts.substitute, &opts.path)?;
            if prompt_user() {
                replace_in_file(&opts.query, &opts.substitute, &opts.path)?;
                return Ok(());
            }
        }
        for file in list_files_in_folder(&opts.path)? {
            display_changes_in_file(&opts.query, &opts.substitute, file.to_str().unwrap())?;
        }

        if prompt_user() {
            return Ok(());
        }
        for file in list_files_in_folder(&opts.path)? {
            replace_in_file(&opts.query, &opts.substitute, file.to_str().unwrap())?;
        }
        return Ok(());
    }

    if !opts.recursive {
        display_changes_in_file(&opts.query, &opts.substitute, &opts.path)?;
        replace_in_file(&opts.query, &opts.substitute, &opts.path)?;
        return Ok(());
    }
    let files = list_files_in_folder(&opts.path)?;
    for file in files {
        match file.to_str() {
            Some(file) => {
                display_changes_in_file(&opts.query, &opts.substitute, file)?;
                replace_in_file(&opts.query, &opts.substitute, file)?;
            },
            None => println!("Could not convert path to string: {:?}", file),
        }
    }
    
    Ok(())
}
