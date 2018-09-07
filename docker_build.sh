#!/bin/bash

mkdir -p ./tmp_bin
cp ./target/x86_64-unknown-linux-musl/release/directemar_crawler ./tmp_bin

docker build ./tmp_bin -f ./Docker/Dockerfile -t rcastill/directemar-crawler

rm -rf ./tmp_bin

