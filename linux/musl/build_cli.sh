cd $(dirname $(dirname $0))
set -ex

export NUM_JOBS=$(nproc)

export OPENSSL_STATIC=1
export RUSTFLAGS='-C link-args=--static'
cargo build --target-dir=./target --target=x86_64-unknown-linux-musl --features=full --profile cli
