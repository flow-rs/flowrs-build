# Setting Environment Variables
export RUSTFLAGS="-C instrument-coverage"
# export RUSTDOCFLAGS="-C instrument-coverage -Z unstable-options --persist-doctests target/debug/doctestbins"

cargo clean
cargo build
cargo test
llvm-profdata merge -sparse default_*.profraw -o coverage.profdata
rm *.profraw

# Run cargo test and parse the output
cargo test --tests --no-run --message-format=json > cargo_output.txt

# Extracting executable file paths from the output
grep "\"executable\":\"" cargo_output.txt | grep -v "null" | sed -E 's/.*"executable":"([^"]+)".*/\1/' > executable_files.txt

# Check if executable_files.txt is empty
if [ ! -s executable_files.txt ]; then
    echo "No executable files found for coverage."
    exit 1
fi

# Prepare arguments for llvm-cov
llvmCovArgs=()
while IFS= read -r file; do
    llvmCovArgs+=("-object" "$file")
done < executable_files.txt

rm executable_files.txt
rm cargo_output.txt

# Run llvm-cov report
llvm-cov export "${llvmCovArgs[@]}" --instr-profile=coverage.profdata --format=lcov -sources ./src/ > coverage.lcov
