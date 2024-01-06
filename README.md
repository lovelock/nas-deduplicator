# This repo is for learning, use this https://github.com/sreedevk/deduplicator in your product environment.

## 背景

群晖相册里面，由于很多次的备份，稳妥起见，所以有很多文件重复备份了很多次，从很多不同设备里面备份的。所以相册看起来比较混乱。

## 目标

相同的文件只保留一份，把重复的文件统一放在一个地方。

## 快速开始

### 预编译的二进制文件

```bash
wget  dedup-${od}-${arch}.tar.xz
tar xjv dedup-${od}-${arch}.tar.xz
./dedup <FROM_PATH> <TO_PATH>
```

### 从源码编译

```bash
git clone https://github.com/lovelock/nas-deduplicator.git
cd nas-deduplicator
cargo build --release
./target/release/dedup <FROM_PATH> <TO_PATH>
```


## 测试用例

```
photos
    - a
    - b （和a相同，覆盖相同文件在同一个目录下的情况）
    - c （没有和别的文件相同）
    - d
        - a （和上级目录里的a/b都相同）
```

把重复的文件放在 dups

```
photos/a 和 photos/d/a 这两个文件是完全相同的，为了防止有hash碰撞，dups/a 就只剩一个文件，另外一个就丢了，万一发生了hash碰撞，就真的丢了一个文件。

photos/a => dups/a
photos/d/a => dups/d/a
```

## 设计思路

1. 遍历整个相册目录，为每个文件生成一个hash，sha256
2. 当遇到一个新文件，生成一个新的hash，把它放在redis里，counter
3. 接下来在继续遍历其他文件时，会直接把counter++，然后检查counter的值
   - 如果counter > 1，就获取这个文件相对于photos目录的相对路径，整个用fs::rename移动到dups目录
   - 如果counter == 1，那就还保留在原来的位置
