use std::fs;
use std::path::Path;

use regex::Regex;

use ratatui::{
    Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, text::Text, widgets::{Block, Borders, Paragraph}
};
use crossterm::{event::{self, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?; // включаем raw mode для корректной работы TUI
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut inputs = vec![
        String::new(), // path
        String::new(), // included extensions
        String::new(), // excluded dirs/files
        String::new(), // regex search
        String::new(), // regex replace
    ];

    let labels = vec![
        "Введите путь к директории (например src):",
        "Введите включённые расширения файлов (через запятую, например swift,txt):",
        "Введите исключаемые директории/файлы (через запятую, например private,tmp):",
        "Введите регулярку поиска (пример: Constants\\.c(\\d+)\\.rawValue):",
        "Введите регулярку замены (пример: Constants.c#$1#.rawValue, $1 = первая capture группа):",
    ];

    let mut current_field = 0;

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    labels.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>()
                )
                .split(size);

            for (i, chunk) in chunks.iter().enumerate() {
                let title = if i == current_field {
                    format!("> {}", labels[i]) 
                } else {
                    labels[i].to_string()
                };
                let paragraph = Paragraph::new(Text::from(inputs[i].as_str()))
                    .block(Block::default().title(title).borders(Borders::ALL));
                f.render_widget(paragraph, *chunk);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => inputs[current_field].push(c),
                KeyCode::Backspace => { inputs[current_field].pop(); },
                KeyCode::Enter => {
                    if current_field == inputs.len() - 1 {
                        break;
                    } else {
                        current_field += 1;
                    }
                },
                KeyCode::Up => {
                    if current_field > 0 { current_field -= 1; }
                },
                KeyCode::Down => {
                    if current_field < inputs.len() - 1 { current_field += 1; }
                },
                KeyCode::Esc => {
                    terminal.clear()?;
                    return Ok(())
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?; 
    terminal.clear()?; 

    let path = &inputs[0];
    let included_extensions: Vec<&str> = inputs[1].split(',').map(|s| s.trim()).collect();
    let excluded_paths: Vec<&str> = inputs[2].split(',').map(|s| s.trim()).collect();
    let regex_search = Regex::new(&inputs[3])?;
    let regex_replace = &inputs[4];

    // println!("{} {?:} {?:} {} {}", path, included_extensions, excluded_paths, regex_search, regex_replace);

    let paths = walk_dir(path, &included_extensions, &excluded_paths);
    find_and_replace_with_replacement(&paths, &regex_search, regex_replace);

    terminal.clear()?;
    println!("Готово!");

    Ok(())
}

fn walk_dir<P: AsRef<Path>>( // типа PathConvertible- что угодно что возвращается в Path
    path: P,
    included_extensions: &[&str],
    excluded_paths: &[&str],
) -> Vec<String> {
    let mut paths = Vec::new();
    let path = path.as_ref();

    if let Ok(dir_iter) = fs::read_dir(path) {
        dir_iter
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|name| !excluded_paths.contains(&name))
                    .unwrap_or(false)
            })
            .for_each(|path| {
                if path.is_dir() {
                    paths.extend(walk_dir(&path, included_extensions, excluded_paths));
                } else if path.is_file() {
                    if path.extension()
                        .and_then(|e| e.to_str())
                        .map(|ext| included_extensions.contains(&ext))
                        .unwrap_or(false)
                    {
                        paths.push(path.to_string_lossy().into_owned());
                    }
                }
            });
    }
    paths
}

fn find_and_replace_with_replacement(paths: &[String], regex: &Regex, replacement: &str) {
    for path in paths {
        match fs::read_to_string(path) {
            Ok(contents) => {
                let updated = regex.replace_all(&contents, replacement);
                if updated == contents { continue; }
                if let Err(err) = fs::write(path, updated.as_bytes()) {
                    eprintln!("Не удалось записать {}: {}", path, err);
                }
            }
            Err(err) => {
                eprintln!("Не удалось прочитать {}: {}", path, err);
            }
        }
    }
}