cd $(dirname $0)
set -ex

emcmake cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build/ --target webrogue -j
cp build/webrogue.wasm build/webrogue.js build/webrogue.data root
