use std::{path::Path, str::FromStr};

use async_channel::Sender;
use async_recursion::async_recursion;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pattern = &args[1];
    let root = Path::new(&args[2]);

    let (s, r) = async_channel::unbounded();

    tokio_scoped::scope(|scope| {
        scope.spawn(async move {
            send_job(root, s).await;
        });

        for _ in 0..10 {
            let r = r.clone();
            scope.spawn(async move {
                while let Ok(msg) = r.recv().await {
                    lookup(Path::new(&msg), pattern).await;
                }
            });
        }
    });
}

async fn lookup(file: &Path, pattern: &str) {
    let mut f = tokio::fs::File::open(file).await.unwrap();
    let mut text = String::new();
    if f.read_to_string(&mut text).await.is_err() {
        return;
    }

    let mut stdout = tokio::io::stdout();
    for line in text.lines() {
        if line.contains(pattern) {
            stdout.write_all(line.as_bytes()).await.unwrap();
        }
    }
}

#[async_recursion]
async fn send_job(root: &Path, s: Sender<String>) {
    let result = tokio::fs::metadata(root).await;
    if result.is_err() {
        return;
    }

    let md = result.unwrap();
    if md.is_symlink() {
        return;
    }

    if md.is_file() {
        s.send(String::from_str(root.to_str().unwrap()).unwrap())
            .await
            .unwrap();
        return;
    }
    if md.is_dir() {
        let mut childs = tokio::fs::read_dir(root).await.unwrap();
        while let Ok(Some(child)) = childs.next_entry().await {
            send_job(&child.path(), s.clone()).await;
        }
    }
}
