use crossterm::style::Stylize;

fn decorate_file_content(file_name: String, content: Vec<String>) -> Vec<String> {
    let mut decorated = vec![];
    decorated.push(format!("╭ {}", file_name));
    if content.is_empty() {
        decorated.push("│ // empty".to_string());
    } else {
        for (i, line) in content.iter().enumerate() {
            decorated.push(format!("│ {} {}", (i + 1).to_string().stylize().dark_grey(), line));
        }
    }
    decorated.push("╰────────".to_string());
    decorated
}

#[test]
fn handle_empty() {
    let file_name = "file.txt".to_string();
    let content = vec![];
    assert_eq!(decorate_file_content(file_name, content), vec![
        "╭ file.txt".to_string(),
        "│ // empty".to_string(),
        "╰────────".to_string()
    ]);
}

#[test]
fn handle_non_empty() {
    let file_name = "file.txt".to_string();
    let content = vec!["line 1".to_string(), "line 2".to_string()];
    assert_eq!(decorate_file_content(file_name, content), vec![
        "╭ file.txt".to_string(),
        "│ \u{1b}[38;5;8m1\u{1b}[39m line 1".to_string(),
        "│ \u{1b}[38;5;8m2\u{1b}[39m line 2".to_string(),
        "╰────────".to_string()
    ]);
}
