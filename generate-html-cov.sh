#!/bin/sh

echo "Installing grcov if not avaliable"
cargo install grcov

echo "Ensuring llvm-tools-preview is avalable and installed"
rustup component add llvm-tools-preview

echo "Ensuring test database is live"
sudo docker-compose up -d

echo "Running tests with instrument coverage"
LLVM_PROFILE_FILE="ipv8-auth-%p-%m.profraw" RUSTFLAGS=-Cinstrument-coverage DATABASE_URL=postgres://postgres:postgres@localhost:5433/postgres cargo test

echo "Generating Coverage Report"
grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing --ignore "/*" -o cov/

echo "Cleaning up profraw files"
rm *.profraw

echo "Done! Open cov/index.html to view results."
