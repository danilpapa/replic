use std::fs;
use std::path::Path;

use regex::Regex;

fn main() {
    let included_extensions: Vec<&str> = vec!["swift", "txt"];
    let excluded_paths: Vec<&str> = vec!["private"];
    let re = Regex::new(r"Constants\.c(\d+)\.rawValue").unwrap();  

    let paths = walk_dir("src", &included_extensions, &excluded_paths);
    find_and_replace(&paths, &re);
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

fn find_and_replace(paths: &Vec<String>, regex: &Regex) {
    for path in paths {
        match fs::read_to_string(&path) {
            Ok(contents) => {
                let updated = regex.replace_all(
                    &contents,
                    "Constants.c$1"
                );
                if updated == contents { continue; }

                if let Err(writing_error) = fs::write(path, updated.as_bytes()) {
                    eprintln!("Не удалось записать {}: {}", path, writing_error);
                }
            }
            Err(err) => {
                eprintln!("Не удалось прочитать {}: {}", path, err);
            }
        }
    }
}