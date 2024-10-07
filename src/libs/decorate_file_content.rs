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
