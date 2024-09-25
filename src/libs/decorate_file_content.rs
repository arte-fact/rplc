use crossterm::style::{Color, Stylize};

pub fn happend_changes_in_file(
    lines: Vec<String>,
    query: &str,
    substitute: &str,
) -> (Vec<String>, usize) {
    let mut decorated = vec![];
    let mut skipped = false;
    let mut changes = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.contains(&query) {
            skipped = false;
            let mut line = line.replace(
                &query,
                &format!("{}", substitute.stylize().with(Color::Green).bold()),
            );

            changes += line.matches(&substitute).count();
            line = format!(
                "{: >4} {: <80}",
                (i + 1).to_string().stylize().with(Color::DarkGrey),
                line
            );

            decorated.push(line);
        } else {
            if !skipped {
                decorated.push(
                    "..."
                        .to_string()
                        .stylize()
                        .with(Color::DarkGrey)
                        .to_string(),
                );
                skipped = true;
            }
        }
    }
    (decorated, changes)
}

pub fn decorate_file_content(
    file_name: &str,
    content: Vec<String>,
    footer_content: &str,
) -> Vec<String> {
    let mut decorated = vec![];
    decorated.push(format!("╭ {}", file_name));
    if content.is_empty() {
        decorated.push("│ // empty".to_string());
    } else {
        for line in content.iter() {
            decorated.push(format!("│ {}", line));
        }
    }
    decorated.push(format!("╰──────── {}", footer_content));
    decorated
}

#[test]
fn handle_empty() {
    let file_name = "file.txt";
    let content = vec![];
    assert_eq!(
        decorate_file_content(file_name, content, "test"),
        vec![
            "╭ file.txt".to_string(),
            "│ // empty".to_string(),
            "╰──────── test".to_string()
        ]
    );
}

#[test]
fn handle_non_empty() {
    let file_name = "file.txt";
    let content = vec!["line 1".to_string(), "line 2".to_string()];
    assert_eq!(
        decorate_file_content(file_name, content, "test"),
        vec![
            "╭ file.txt".to_string(),
            "│ line 1".to_string(),
            "│ line 2".to_string(),
            "╰──────── test".to_string()
        ]
    );
}

#[test]
fn handle_changes() {
    let lines = vec!["line 1".to_string(), "line 2".to_string(), "line 3".to_string()];
    let query = "line";
    let substitute = "test";
    assert_eq!(
        happend_changes_in_file(lines, query, substitute),
        (
            vec![
                "\u{1b}[38;5;8m1\u{1b}[39m \u{1b}[38;5;10m\u{1b}[1mtest\u{1b}[0m 1".to_string(),
                "\u{1b}[38;5;8m2\u{1b}[39m \u{1b}[38;5;10m\u{1b}[1mtest\u{1b}[0m 2".to_string(),
                "\u{1b}[38;5;8m3\u{1b}[39m \u{1b}[38;5;10m\u{1b}[1mtest\u{1b}[0m 3".to_string(),
            ],
            3
        )
    );
}
