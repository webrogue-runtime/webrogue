set -ex

cd $(dirname $0)

cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --target webrogue --parallel 
cpack --config build/CPackConfig.cmake 
mv webrogue-*-Linux.deb webrogue.deb
mv webrogue-*-Linux.rpm webrogue.rpm

rm -rf _CPack_Packages
