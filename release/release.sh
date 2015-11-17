#! /bin/bash
cargo build --release
mv ./target/release/git-clean release/git-clean
