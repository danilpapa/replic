use std::fs;
use std::path::Path;

fn main() {
    let included_extensions: Vec<&str> = vec!["swift", "txt"];
    let excluded_paths: Vec<&str> = vec!["private"];

    let result = walk_dir("src", &included_extensions, &excluded_paths);
    for path in result {
        println!("{}", path);
    }
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