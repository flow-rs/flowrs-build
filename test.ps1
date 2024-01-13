# Setting Environment Variables
$Env:RUSTFLAGS = "-C instrument-coverage"
#$Env:RUSTDOCFLAGS = "-C instrument-coverage -Z unstable-options --persist-doctests target/debug/doctestbins"

rustup component add llvm-tools-preview
cargo install grcov

cargo clean
cargo build
cargo test


# Run cargo test and parse the output
$cargoTestOutput = cargo test --tests --no-run --message-format=json | ConvertFrom-Json
$testFiles = $cargoTestOutput | Where-Object { $_.profile.test -eq $true } | Select-Object -ExpandProperty filenames

# Filter out unwanted files and prepare arguments for llvm-cov
$llvmCovArgs = @()
foreach ($file in $testFiles) {
    if (-not $file.Contains("dSYM") -and -not $file.EndsWith(".pdb")) {
        $llvmCovArgs += "-object", $file
    }
}
# Generate HTML report using grcov
& grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing --ignore "/*" -o ./target/debug/coverage/

llvm-profdata merge -sparse default_*.profraw -o coverage.profdata
Get-ChildItem -Filter *.profraw | Remove-Item

# Generate lcov report
& llvm-cov export @llvmCovArgs --instr-profile=coverage.profdata --format=lcov -sources ./src/ | Out-File -Encoding utf8 ./target/debug/coverage/lcov.info

Get-ChildItem -Filter *.profdata | Remove-Item