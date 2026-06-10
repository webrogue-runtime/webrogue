cd $(dirname $(dirname $0))
set -ex

export NUM_JOBS=$(nproc)

cargo build --target-dir=./gnu/$ARCH/target --target=$ARCH-unknown-linux-gnu --features=full --profile cli
