@echo off

mkdir "flowrs-dependencies"
cd "%CD%\flowrs-dependencies"
git "clone" "https://github.com/flow-rs/flowrs.git"
cd "%CD%\flowrs"
git "fetch" "--all"
git "checkout" "-b" "dev" "--track" "origin\dev"
cd ".."
git "clone" "https://github.com/flow-rs/flowrs-std.git"
cd "%CD%\flowrs-std"
git "fetch" "--all"
git "checkout" "-b" "dev" "--track" "origin\dev"