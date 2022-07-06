use std::fs::{metadata, read_dir};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pattern = &args[1];
    let root = Path::new(&args[2]);
    recursive(root, pattern)
}

fn lookup(file: &Path, pattern: &str) {
    let result = std::fs::read_to_string(file);
    if result.is_err() {
        return;
    }

    for line in result.unwrap().lines() {
        if line.contains(pattern) {
            println!("{:?}", line);
        }
    }
}

fn recursive(root: &Path, pattern: &str) {
    let result = metadata(root);
    if result.is_err() {
        return;
    }

    let md = result.unwrap();
    if md.is_symlink() {
        return;
    }
    if md.is_file() {
        return lookup(root, pattern);
    }
    if md.is_dir() {
        let childs = read_dir(root).unwrap();
        for child in childs.flatten() {
            recursive(&child.path(), pattern);
        }
    }
}
