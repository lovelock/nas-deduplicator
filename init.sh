#!/usr/bin/env bash


echo "cleaning up"
echo 'flushdb' | redis-cli -h 127.0.0.1 -p 6379
rm -rf /tmp/dedup/

echo "rebuilding test cases"
mkdir -p /tmp/dedup/{photos,dups} && cd /tmp/dedup/photos

echo 'aaaa' > a
echo 'aaaa' > b
echo 'cccc' > c
mkdir d
echo 'aaaa' > d/a
mkdir 'my photos'
echo 'aaaa' > my\ photos/a

echo "done"
