# `walk_dir`

## Назначение

Функция `walk_dir` рекурсивно обходит директорию и собирает **все файлы**, которые соответствуют указанным условиям:

1. Расширение файла есть в списке `included_extensions`.
2. Файл или директория **не находится** в списке `excluded_paths`.

Возвращает **вектор строковых путей** ко всем подходящим файлам.

---

## Сигнатура функции

```rust
fn walk_dir<P: AsRef<Path>>(
    path: P,
    included_extensions: &[&str],
    excluded_paths: &[&str],
) -> Vec<String>
````

**Пояснение параметров:**

* `path: P` — путь к директории, с которой начинается обход. Любой тип, который можно преобразовать в `Path` (`String`, `&str`, `PathBuf` и т.д.).
* `included_extensions: &[&str]` — массив расширений файлов, которые нужно включить в результат, например `["swift", "txt"]`.
* `excluded_paths: &[&str]` — массив имён папок или файлов, которые нужно пропускать, например `["private", "tmp"]`.

**Возвращаемое значение:** `Vec<String>` — список файлов, подходящих под условия.

---

## Как работает функция

1. Создаём пустой вектор `paths` для хранения результатов.

2. Преобразуем `path` к типу `&Path` с помощью `path.as_ref()`.

3. Читаем содержимое директории через `fs::read_dir(path)`:

   ```rust
   if let Ok(dir_iter) = fs::read_dir(path) {
   ```

   * `read_dir` возвращает итератор по элементам (файлам и папкам).
   * Если директорию нельзя открыть, функция вернёт пустой список.

4. Для каждого элемента директории:

   ```rust
   dir_iter
       .flatten()                 // игнорируем ошибки чтения отдельных файлов
       .map(|entry| entry.path()) // получаем полный путь
   ```

5. Фильтруем элементы по `excluded_paths`:

   ```rust
   .filter(|path| {
       path.file_name()
           .and_then(|n| n.to_str())
           .map(|name| !excluded_paths.contains(&name))
           .unwrap_or(false)
   })
   ```

   * `file_name()` возвращает имя файла/папки.
   * `to_str()` преобразует его в строку.
   * Если имя есть в `excluded_paths`, элемент пропускается.

6. Обрабатываем каждый путь с помощью `for_each`:

   ```rust
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
   ```

   * Если элемент — директория: вызываем **рекурсивно** `walk_dir` для обхода её содержимого. Результат добавляется в `paths`.
   * Если элемент — файл:

     * Получаем расширение файла через `path.extension()`.
     * Сравниваем с `included_extensions`.
     * Если совпадает, добавляем в `paths` (строку пути через `to_string_lossy()`).

7. В конце функция возвращает весь список файлов:

   ```rust
   paths
   ```

---

## Пример использования

```rust
let included = vec!["swift", "txt"];
let excluded = vec!["private", "tmp"];
let files = walk_dir("src", &included, &excluded);

for file in files {
    println!("{}", file);
}
```

**Пример результата:**

```
src/main.swift
src/lib/utils.txt
src/data/file1.swift
```

---

## Особенности и советы

* Рекурсивный обход — функция обходит все вложенные папки.
* Использует стандартную библиотеку Rust (`std::fs`, `std::path`) — не требует сторонних зависимостей.
* Обрабатывает ошибки отдельных файлов с помощью `.flatten()` и `.unwrap_or(false)`.
* Работает с любыми типами пути благодаря `AsRef<Path>`.

---

## Полезные ссылки

* [Документация std::fs](https://doc.rust-lang.org/std/fs/)
* [Документация std::path](https://doc.rust-lang.org/std/path/)
* [Документация Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html)