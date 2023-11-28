@echo off
cargo clean
cargo build
@REM cargo tarpaulin
@REM grcov . -s . --binary-path ./target/debug/ --branch -t lcov -o ./target/debug/lcov
SET RUSTFLAGS=-C instrument-coverage
SET RUSTDOCFLAGS=-C instrument-coverage -Z unstable-options --persist-doctests target/debug/doctestbins
cargo test
llvm-profdata merge -sparse default_*.profraw -o coverage.profdata