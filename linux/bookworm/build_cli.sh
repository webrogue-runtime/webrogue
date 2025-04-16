cd $(dirname $(dirname $0))
set -ex

export NUM_JOBS=$(nproc)

# rustup target add x86_64-unknown-linux-gnu
cargo build --target-dir=./target --target=x86_64-unknown-linux-gnu --features=full --profile cli
