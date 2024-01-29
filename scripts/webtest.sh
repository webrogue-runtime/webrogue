# docker run --rm -v $(pwd):/src -u $(id -u):$(id -g) emscripten/emsdk 

set -ex

sh platforms/Web/build_and_pack.sh Release

cd platforms/Web/homepage
bundle exec jekyll serve
