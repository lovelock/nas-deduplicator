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
use log::{info, warn};

#[derive(Debug, Parser)]
struct Cli {
    from_path: String,
    to_path: String,
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let from = PathBuf::from(&args.from_path);
    let to = PathBuf::from(&args.to_path);
    walk_dir(&from, &to);
}

fn walk_dir(from_path: &Path, to_path: &Path) {
    for entry in WalkDir::new(from_path) {
        let entry = entry.unwrap();
        let this_path = entry.path();
        if this_path.is_dir() || this_path.is_symlink() || this_path.to_str().unwrap().contains("@") {
            warn!("path {} skipped", this_path.to_str().unwrap());
        } else {
            if !first_found(this_path) {
                let relative = this_path.relative_to(from_path).expect("failed to get the relative path");

                let mut b = to_path.join(relative.as_str());
                b.pop();
                create_dir_all(b.to_str().unwrap()).expect("failed to create dir");

                let from_path = this_path.to_owned();
                let dest_path = to_path.join(relative.as_str());

                info!("from path: {}", from_path.to_str().unwrap());
                info!("dest path: {}", dest_path.to_str().unwrap());
                if !dest_path.parent().unwrap().exists() {
                    info!("creating {}", dest_path.parent().unwrap().to_str().unwrap().trim());
                    create_dir_all(dest_path.parent().unwrap()).expect("Create dir failed");
                }

                fs::rename(from_path.clone(), dest_path).unwrap_or_else(|_| { panic!("{}", format!("move file {} failed", from_path.to_str().unwrap().trim()).as_str().trim().to_string()) });

                let mut cwd = PathBuf::from(this_path.to_str().unwrap());
                cwd.pop();
                info!("checking if the cwd is empty {}", cwd.to_str().unwrap().trim());
                if cwd.read_dir().unwrap().next().is_none() {
                    warn!("the {} is empty and will remove it", cwd.to_str().unwrap().trim());
                    remove_dir(cwd).unwrap();
                }
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

    let count: i32 = conn.incr(hash.clone(), 1)
        .unwrap_or_else(|_| panic!("failed to execute INCR for {}", path.to_str().unwrap().trim()));

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