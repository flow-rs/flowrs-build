#!/bin/bash

# Setting Environment Variables
export RUSTFLAGS="-C instrument-coverage"
# Uncomment the line below if needed
# export RUSTDOCFLAGS="-C instrument-coverage -Z unstable-options --persist-doctests target/debug/doctestbins"

cargo clean
cargo build
cargo test

# Run cargo test and parse the output
cargo test --tests --no-run --message-format=json > cargo_output.json

# Filter out unwanted files and prepare arguments for llvm-cov
llvmCovArgs=()
while read -r file; do
    if [[ ! $file =~ "dSYM" ]] && [[ ! $file =~ ".pdb" ]] && [[ ! $file =~ ".rmeta" ]]; then
        llvmCovArgs+=("-object" "$file")
    fi
done < <(grep -o '"filenames":\["[^"]*' cargo_output.json | awk -F '["]' '{print $4}')

# Generate HTML report using grcov
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing --ignore "/*" -o ./target/debug/coverage/

# Merge profraw files into a single profdata file
xcrun llvm-profdata merge -sparse default_*.profraw -o coverage.profdata
rm *.profraw

# Generate lcov report
xcrun llvm-cov export "${llvmCovArgs[@]}" --instr-profile=coverage.profdata --format=lcov > ./target/debug/coverage/lcov.info


# Clean up
rm *.profdata
rm cargo_output.json