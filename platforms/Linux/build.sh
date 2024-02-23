set -ex

cd $(dirname $0)

cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DWEBROGUE_APPIMAGE=OFF
cmake --build build --target webrogue --parallel 
cpack --config build/CPackConfig.cmake && mv webrogue-*-Linux.deb webrogue.deb
rm -rf _CPack_Packages

cmake -S . -B build/ -DCMAKE_BUILD_TYPE=Release -DWEBROGUE_APPIMAGE=ON
cmake --build build/ --target webrogue --parallel 
cmake --install build/ --prefix .
readelf -d build/webrogue | grep NEEDED
