use std::os::unix::process::CommandExt;
use std::{fs, process::Command};
use std::path::Path;

use regex::Regex;

use ratatui::{
    Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, text::Text, widgets::{Block, Borders, Paragraph}
};
use crossterm::{event::{self, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
    Этот блок кода создаёт текстовый пользовательский интерфейс (TUI)
    в терминале с помощью Rust и библиотек RatatUI (для визуальных компонентов) 
    и Crossterm (для работы с терминалом и событиями клавиатуры). 
    
    Он включает пять интерактивных полей ввода: 
        путь к директории, 
        включённые расширения файлов, 
        исключаемые директории/файлы, 
        регулярное выражение для поиска и регулярное выражение для замены. 
    
    Функция enable_raw_mode() переводит терминал в режим "raw", 
    позволяя отслеживать нажатия клавиш в реальном времени, 
    а Terminal с CrosstermBackend обеспечивает рендеринг интерфейса.
    
    Поля ввода выделяются, поддерживаются клавиши навигации (стрелки вверх/вниз), 
    редактирование текста (Backspace, ввод символов) и подтверждение (Enter). 
    
    Внизу экрана отображается подсказка для пользователя с инструкциями.
    */
    enable_raw_mode()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut inputs = vec![
        String::new(),
        String::new(), 
        String::new(), 
        String::new(), 
        String::new(),
    ];
    let labels = vec![
        "Entet_path_to_your_directory_FE:_src/data:",
        "Enter_INCLUDED_files_extension_comma_separated,_FE:_swift,txt:",
        "Enter_EXCLUDED_directory_names_comma_separated,_FE:_private,design:",
        "Enter_search_regex_FE:_Constants\\.c(\\d+)\\.rawValue:",
        "Enter_replace_regex_FE:_Constants.c#$1#.rawValue,_$1_=_first capture group):",
    ];
    let mut current_field = 0;

    loop {
        terminal.draw(|f| {
            let size = f.area();

            let mut constraints = labels.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>();
            constraints.push(Constraint::Min(3));
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(&constraints) 
                .split(size);

            for (i, chunk) in chunks.iter().enumerate() {
                if i < labels.len() {
                    let title = if i == current_field { format!("> {}", labels[i]) } else { labels[i].to_string() };
                    let paragraph = Paragraph::new(Text::from(inputs[i].as_str()))
                        .block(Block::default().title(title).borders(Borders::ALL));
                    f.render_widget(paragraph, *chunk);
                } else {
                    let help_text = "Перед использованием ознакомьтесь с документацией в src/doc/doc.md\n\
                    для работы со скриптом и регулярными выражениями.";
                    let help_paragraph = Paragraph::new(Text::from(help_text))
                        .block(Block::default().borders(Borders::ALL).title("Подсказка"));
                    f.render_widget(help_paragraph, *chunk);
                }
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
            KeyCode::Up => { if current_field > 0 { current_field -= 1; } },
            KeyCode::Down => { if current_field < inputs.len() - 1 { current_field += 1; } },
            KeyCode::Esc => { 
                disable_raw_mode()?;
                clear_term();
                reset();
                return Ok(())
            },
            _ => {}
            }
        }       
    }

    clear_term();

    let path = &inputs[0];
    let included_extensions: Vec<&str> = inputs[1].split(',').map(|s| s.trim()).collect();
    let excluded_paths: Vec<&str> = inputs[2].split(',').map(|s| s.trim()).collect();
    let regex_search = Regex::new(&inputs[3])?;
    let regex_replace = &inputs[4];

    let paths = walk_dir(path, &included_extensions, &excluded_paths);
    find_and_replace_with_replacement(&paths, &regex_search, regex_replace);

    clear_term();
    println!("Готово!");

    disable_raw_mode()?;
    reset();
    Ok(())
}

/// Рекурсивно обходит указанную директорию и собирает все файлы, которые:
/// 1. Имеют расширение, указанное в `included_extensions`.
/// 2. Не находятся в директориях, перечисленных в `excluded_paths`.
///
/// Использует стандартную библиотеку Rust (`std::fs`, `std::path`) для работы с файловой системой.
/// `AsRef<Path>` позволяет передавать любые типы, которые могут быть преобразованы в `Path`.
/// Возвращает вектор строковых путей ко всем подходящим файлам.
fn walk_dir<P: AsRef<Path>>(
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

/// Проходит по списку файлов `paths`, ищет в них все вхождения регулярного выражения `regex`
/// и заменяет их на строку `replacement`.
///
/// Использует стандартную библиотеку `std::fs` для чтения и записи файлов и crate `regex`
/// для поиска по регулярному выражению. Если файл не может быть прочитан или записан,
/// выводит сообщение об ошибке в stderr.
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

/// Очищает терминал, выполняя команду `clear` через Bash.
fn clear_term() {
    let _ = Command::new("bash")
        .arg("-c")
        .arg("clear")
        .status();
}

fn reset() {
    let _ = Command::new("bash")
        .arg("-c")
        .arg("reset")
        .status();
}