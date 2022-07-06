use crossbeam::channel::{unbounded, Sender};
use crossbeam::thread::{self, ScopedJoinHandle};
use std::{env, io::Read, path::Path, str::FromStr};

fn main() {
    let args: Vec<String> = env::args().collect();
    let pattern = &args[1];
    let root = Path::new(&args[2]);
    let (s, r) = unbounded();

    thread::scope(|scope| {
        let handle = scope.spawn(move |_| send_job(root, s));

        let mut handles: Vec<ScopedJoinHandle<()>> = Vec::with_capacity(11);
        handles.push(handle);
        for _ in 0..10 {
            let r = r.clone();
            let handle = scope.spawn(move |_| {
                while let Ok(msg) = r.recv() {
                    lookup(Path::new(&msg), pattern)
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }
    })
    .unwrap();
}

fn lookup(file: &Path, pattern: &str) {
    let mut f = std::fs::File::open(file).unwrap();
    let mut text = String::new();
    let result = f.read_to_string(&mut text);
    if result.is_err() {
        return;
    }

    for line in text.lines() {
        if line.contains(pattern) {
            println!("{:?} {:?} {:?}", pattern, file.to_str().unwrap(), line)
        }
    }
}

fn send_job(root: &Path, s: Sender<String>) {
    let result = std::fs::metadata(root);
    if result.is_err() {
        return;
    }

    let md = result.unwrap();
    if md.is_symlink() {
        return;
    }
    if md.is_file() {
        s.send(String::from_str(root.to_str().unwrap()).unwrap())
            .unwrap();
        return;
    }
    if md.is_dir() {
        let childs = std::fs::read_dir(root).unwrap();
        for child in childs.flatten() {
            send_job(&child.path(), s.clone());
        }
    }
}
