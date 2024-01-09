# This repo is for learning, use this https://github.com/sreedevk/deduplicator in your product environment.

## Background

There are lots of duplicate files in Photos directory, which makes a bad experience of Synology Photos.

## Goal

One file is saved only once, and all the other duplicates should be moved and then deleted (if necessary).

## Quick Start

### with pre-built binary

```bash
wget dedup-aarch64-apple-darwin.tar.xz 
tar xjv dedup-aarch64-apple-darwin.tar.xz 
./dedup <FROM_PATH> <TO_PATH>
```

### build from source

```bash
git clone https://github.com/lovelock/nas-deduplicator.git
cd nas-deduplicator
cargo build --release
./target/release/dedup <FROM_PATH> <TO_PATH>
```


## Test cases

photos is the directory where original files are stored and duplicates exist.

```
photos
    - a
    - b (has the same content with a, covers the case where multiple files with the same content but don't share the same name)
    - c (with nothing to do with a)
    - d
        - a (has the same content with ../a and ../b, covers the case where another file in another directory with the same content and name)
```

dups is the directory where the duplicates found out from photos should be stored. **The relative path to photos is reserved in dups**, both a and d/a would be moved to dups, nothing is lost.

```
photos/a and photos/d/a are reserved to dups

photos/a => dups/a
photos/d/a => dups/d/a
```

## Design

1. Scan the directory for files and generate a hash for every file.
2. Put the hash of a file to redis counter
3. The counter would increase by 1 in any case and then check the value of the counter
   - counter > 1 indicates the file was found before which should be moved to dups
   - counter == 1 indicates the file was a first found and nothing further should be done

## FAQ

1. Q: Why do I have to deploy a redis server instead of using a map in the program?
   A: Imagine there are plenty of directories to scan, you scan A today and B in another day, as long as the redis server is alive, you'll be able to get a global unique file.
