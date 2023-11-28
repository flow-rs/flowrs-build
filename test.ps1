# Setting Environment Variables
$Env:RUSTFLAGS = "-C instrument-coverage"
#$Env:RUSTDOCFLAGS = "-C instrument-coverage -Z unstable-options --persist-doctests target/debug/doctestbins"

cargo clean
cargo build
cargo test
llvm-profdata merge -sparse default_*.profraw -o coverage.profdata
Get-ChildItem -Filter *.profraw | Remove-Item

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

# Run llvm-cov report
#& llvm-cov report @llvmCovArgs --instr-profile=coverage.profdata --summary-only # and/or other options
& llvm-cov export @llvmCovArgs --instr-profile=coverage.profdata --format=lcov  -sources ./src/ > coverage.lcov
