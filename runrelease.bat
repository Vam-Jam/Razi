@ECHO OFF
SET RUSTFLAGS=-C target-cpu=native

cargo run --release