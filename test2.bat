@echo off
setlocal enabledelayedexpansion

set RUSTFLAGS=-C instrument-coverage
set RUSTDOCFLAGS=-C instrument-coverage -Z unstable-options --persist-doctests target/debug/doctestbins

for /f "delims=" %%a in ('cargo test --no-run --message-format=json ^| jq -r "select(.profile.test == true) ^| .filenames[]"') do (
    set "file=%%a"
    if not "!file!"=="!file:dSYM=!" (
        for /R "target/debug/doctestbins/" %%b in (rust_out) do (
            if exist "%%b" (
                set "files=!files! -object %%b"
            )
        )
    )
)

llvm-cov report !files! --instr-profile=json5format.profdata --summary-only
