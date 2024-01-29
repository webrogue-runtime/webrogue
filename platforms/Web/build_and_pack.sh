# docker run --rm -v $(pwd):/src -u $(id -u):$(id -g) emscripten/emsdk 

cd $(dirname $0)

set -ex

BUILD_TYPE=$1

emcmake cmake -B game/build -S game/ -DCMAKE_BUILD_TYPE=$BUILD_TYPE
cmake --build game/build/ --target pack_artifacts -j

rm -rf homepage/game/artifacts
mv ../../artifacts homepage/game

emcmake cmake -B baked_game/build -S baked_game/ -DCMAKE_BUILD_TYPE=$BUILD_TYPE
cmake --build baked_game/build/ --target pack_artifacts -j

mv ../../artifacts/* homepage/backed_game
rmdir ../../artifacts

cd homepage
bundle exec jekyll build
cp -r _site/ ../../../artifacts
cd ..
cp google8fff32979a1b49aa.html ../../artifacts

# cd artifacts
# python3 -m http.server
