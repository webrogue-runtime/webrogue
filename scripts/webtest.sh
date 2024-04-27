# docker run --rm -v $(pwd):/src -u $(id -u):$(id -g) emscripten/emsdk 

set -ex

cd $(dirname $0)/..

emcmake cmake -B platforms/Web/build -S platforms/Web -DCMAKE_BUILD_TYPE=Release
cmake --build platforms/Web/build --target pack_artifacts -j

cd platforms/Web
python3 -m http.server
