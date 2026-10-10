set -ex
cd "$(dirname $0)"

cargo run -- ../../webrogue-sdk/libraries/libwr4c/wr4c.h ../../webrogue-sdk/libraries/libwr4c/wr4c.c ../gfx/wit/webrogue-gfx.wit
