# Документация к TUI на Rust

## Обзор

**Текстовый пользовательский интерфейс (TUI)** для работы с файлами:  
- Позволяет интерактивно вводить параметры для поиска и замены строк в файлах.  
- Использует библиотеки:
  - [`RatatUI`](https://docs.rs/ratatui/latest/ratatui/) — визуальные компоненты TUI (окна, параграфы, блоки, лейауты).
  - [`Crossterm`](https://docs.rs/crossterm/latest/crossterm/) — управление терминалом, обработка событий клавиатуры.
  - [`regex`](https://docs.rs/regex/latest/regex/) — поиск и замена по регулярным выражениям.

---

## Структура скрипта

### 1. TUI с 5 полями ввода

Поля ввода, которые пользователь вводит через терминал:  

1. **Путь к директории** — например `src/data`  
2. **Включённые расширения файлов** — через запятую, например `swift,txt`  
3. **Исключаемые директории или файлы** — через запятую, например `private,tmp`  
4. **Регулярка поиска** — пример: `Constants\.c(\d+)\.rawValue`  
5. **Регулярка замены** — пример: `Constants.c#$1#.rawValue`, где `$1` — первая capture-группа 

**Навигация и редактирование:**
- Стрелки ↑↓ — перемещение между полями
- Backspace — удаление символа
- Enter — подтверждение текущего поля
- Esc — выход из TUI

#### Пример кода TUI:

```rust
enable_raw_mode()?;
let stdout = std::io::stdout();
let backend = CrosstermBackend::new(stdout);
let mut terminal = Terminal::new(backend)?;
let mut inputs = vec![String::new(); 5];
let labels = vec![
    "Enter path to your directory:",
    "Enter INCLUDED file extensions, comma separated:",
    "Enter EXCLUDED directory names, comma separated:",
    "Enter search regex:",
    "Enter replace regex ($1, $2... = capture groups):",
];
````

