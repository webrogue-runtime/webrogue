cd $(dirname $0)
set -ex

# make -C ../examples

emcmake cmake -S . -B build -DCMAKE_BUILD_TYPE=Debug
cmake --build build/ --target webrogue -j
cp build/webrogue.wasm build/webrogue.js root