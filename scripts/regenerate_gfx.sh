set -ex
cd "$(dirname $0)"

cd ..

cargo run \
    --manifest-path crates/events-apigen/Cargo.toml \
    -- \
    webrogue-sdk/libraries/libwr4c/wr4c.h \
    webrogue-sdk/libraries/libwr4c/wr4c.c \
    wit/deps/gfx.wit

wit-bindgen c \
    --out-dir webrogue-sdk/libraries/libwr4c \
    wit \
    --world webrogue-gfx-world \
    --rename-world webrogue-gfx
