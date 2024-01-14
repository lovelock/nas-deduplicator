use fs::{File, rename};
use std::collections::BTreeMap;
use std::fs;
use std::fs::{create_dir_all, remove_dir};
use std::hash::Hasher;
use std::path::{Path, PathBuf};

use clap::Parser;
use log::{info, warn};
use memmap2::Mmap;
use relative_path::PathExt;
use walkdir::WalkDir;

#[derive(Debug, Parser)]
struct Cli {
    from_path: String,
    to_path: String,
}

fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let args = Cli::parse();
    let from = PathBuf::from(&args.from_path);
    let to = PathBuf::from(&args.to_path);
    walk_dir(&from, &to);
}

fn walk_dir(from_path: &Path, to_path: &Path) {
    for entry in WalkDir::new(from_path) {
        let entry = entry.unwrap();
        let this_path = entry.path();
        if this_path.is_dir() {
            warn!("dir {} skipped", this_path.to_str().unwrap());
            continue;
        }
        if this_path.is_symlink() {
            warn!("symlink {} skipped", this_path.to_str().unwrap());
            continue;
        }
        if this_path.to_str().unwrap().contains("@") {
            warn!("synology metadata dir {} skipped", this_path.to_str().unwrap());
            continue;
        }

        let (hash, is_first_found) = first_found(this_path);
        update_records(this_path, hash, is_first_found);

        if !is_first_found {
            let relative = this_path.relative_to(from_path).expect("failed to get the relative path");

            let from_path = this_path.to_owned();
            let dest_path = to_path.join(relative.as_str());

            info!("from path: {}", from_path.to_str().unwrap());
            info!("dest path: {}", dest_path.to_str().unwrap());
            if !dest_path.parent().unwrap().exists() {
                info!("creating {}", dest_path.parent().unwrap().to_str().unwrap().trim());
                create_dir_all(dest_path.parent().unwrap()).expect("Create dir failed");
            }

            rename(from_path.clone(), dest_path).unwrap_or_else(|_| { panic!("{}", format!("move file {} failed", from_path.to_str().unwrap().trim()).as_str().trim().to_string()) });

            let mut cwd = PathBuf::from(this_path.to_str().unwrap());
            cwd.pop();
            info!("checking if the cwd is empty {}", cwd.to_str().unwrap().trim());
            if cwd.read_dir().unwrap().next().is_none() {
                warn!("the {} is empty and will be removed", cwd.to_str().unwrap().trim());
                remove_dir(cwd).unwrap();
            }
        }
    }
}

fn hash(path: &Path) -> String {
    let file = File::open(path).unwrap();
    let mapper = unsafe { Mmap::map(&file).unwrap() };
    let mut primhasher = fxhash::FxHasher::default();

    mapper
        .chunks(1_000_000)
        .for_each(|chunk| primhasher.write(chunk));

    primhasher.finish().to_string()
}

fn first_found(path: &Path) -> (String, bool) {
    let hash = hash(path);
    let mut conn = connect();

    let result: BTreeMap<String, String> = redis::cmd("HGETALL")
        .arg(format!("{}:{}", "dedup", hash.clone()))
        .query(&mut conn)
        .expect(format!("failed to get all elements of {}", path.to_str().unwrap().trim()).as_str());

    info!("result from {} is {:?} ", hash, result);

    if result.len() == 0 {
        return (hash, true);
    }

    let tmp_path = path.to_str().unwrap();
    warn!("tmp_path: {}", tmp_path);

    match result.get(tmp_path) {
        None => {}
        Some(v) => {
            if v.eq("reserved") {
                return (hash, true);
            }
        }
    }

    return (hash, false);
}

fn update_records(path: &Path, hash: String, is_first_found: bool) {
    let mut conn = connect();
    let mut driver: BTreeMap<String, String> = BTreeMap::new();
    if is_first_found {
        driver.insert(path.to_str().unwrap().to_string(), String::from("reserved"));
    } else {
        driver.insert(path.to_str().unwrap().to_string(), String::from("moved"));
    }

    let _: () = redis::cmd("HSET")
        .arg(format!("{}:{}", "dedup", hash))
        .arg(driver)
        .query(&mut conn)
        .expect(format!("failed to update hash of {} => {}", hash, path.to_str().unwrap()).as_str());
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