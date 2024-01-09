#!/usr/bin/env bash


rm -rf /tmp/dedup/dups/*

mkdir -p /tmp/dedup/{photos,dups} && cd /tmp/dedup/photos

echo 'aaaa' > a
echo 'aaaa' > b
echo 'cccc' > c
mkdir d
echo 'aaaa' > d/a

