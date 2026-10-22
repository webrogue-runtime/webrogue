cd "$(dirname "$0")"
set -ex

rm -rf deps
mkdir deps
cp ../../../external/wasmtime/crates/wasi/src/p3/wit/deps/filesystem.wit deps
cp ../../../external/wasmtime/crates/wasi/src/p3/wit/deps/clocks.wit deps


