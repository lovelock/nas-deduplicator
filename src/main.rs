use fs::File;
use std::fs;
use std::fs::{create_dir_all, remove_dir};
use std::hash::Hasher;
use std::path::{Path, PathBuf};

use clap::Parser;
use memmap2::Mmap;
use redis::Commands;
use relative_path::PathExt;
use walkdir::WalkDir;

#[derive(Debug, Parser)]
struct Cli {
    from_path: String,
    to_path: String,
}

fn main() {
    let args = Cli::parse();
    let from = PathBuf::from(&args.from_path);
    let to = PathBuf::from(&args.to_path);
    walk_dir(&from, &to);
}

fn walk_dir(from_path: &Path, to_path: &Path) {
    for entry in WalkDir::new(from_path) {
        let entry = entry.unwrap();
        let this_path = entry.path();
        if !this_path.is_dir() && !first_found(this_path) {
            let relative = this_path.relative_to(from_path).expect("failed to get the relative path");

            let mut b = to_path.join(relative.as_str());
            b.pop();
            create_dir_all(b.to_str().unwrap()).expect("failed to create dir");
            fs::rename(this_path, to_path.join(relative.as_str())).unwrap_or_else(|_| { panic!("{}", format!("move file {} failed", this_path.to_str().unwrap().trim()).as_str().trim().to_string()) });

            let mut cwd = PathBuf::from(this_path.to_str().unwrap());
            cwd.pop();
            println!("{}", cwd.to_str().unwrap().trim());
            if cwd.read_dir().unwrap().next().is_none() {
                remove_dir(cwd).unwrap();
            }
        }
    }
}

fn crypto(path: &Path) -> String {
    let file = File::open(path).unwrap();
    let mapper = unsafe { Mmap::map(&file).unwrap() };
    let mut primhasher = fxhash::FxHasher::default();

    mapper
        .chunks(1_000_000)
        .for_each(|chunk| primhasher.write(chunk));

    primhasher.finish().to_string()
}

fn first_found(path: &Path) -> bool {
    let hash = crypto(path);
    let mut conn = connect();

    let _: () = conn.incr(hash.clone(), 1)
        .unwrap_or_else(|_| panic!("failed to execute INCR for {}", path.to_str().unwrap().trim()));

    let count: i32 = conn.get(hash.clone()).unwrap_or_else(|_| panic!("failed to execute GET for {}", path.to_str().unwrap().trim()));

    println!("path {}, hash {} ", path.to_str().unwrap().trim(), hash.as_str());

    // 如果count == 1，说明不需要移动，如果count > 1，说明需要移动
    count == 1
}


fn connect() -> redis::Connection {
    let redis_host_name = "127.0.0.1:6379";
    let uri_scheme = "redis";

    let redis_conn_url = format!("{}://:{}@{}", uri_scheme, "", redis_host_name);

    redis::Client::open(redis_conn_url)
        .expect("Invalid connection URL")
        .get_connection()
        .expect("failed to connect to Redis")
}