mkdir flowrs-dependencies
cd ./flowrs-dependencies

git clone https://github.com/flow-rs/flowrs.git
cd ./flowrs
git fetch --all
git checkout -b dev --track origin/dev
cargo build
cd ..

git clone https://github.com/flow-rs/flowrs-std.git
cd ./flowrs-std
git fetch --all
git checkout -b dev --track origin/dev
cargo build
cd ..
cd ..
cargo build