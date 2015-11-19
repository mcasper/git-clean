#! /bin/bash

rm -f /usr/local/bin/git-clean
cargo build --release
mv ./target/release/git-clean /usr/local/bin/git-clean
