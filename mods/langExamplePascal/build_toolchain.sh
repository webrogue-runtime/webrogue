cd $(dirname $0)

set -ex

FPC_VERSION=3.3.1
FPC_COMMIT=d809b4ba982e4ac905631054665ff5a973b4d491 # somewhere on main. feel free to update

git clone https://gitlab.com/freepascal.org/fpc/source.git fpc_source && true
rm -rf fpcwasm
cd fpc_source
git checkout $FPC_COMMIT
make all OS_TARGET=wasi CPU_TARGET=wasm32 BINUTILSPREFIX= OPT="-O-" PP=fpc
make crossinstall OS_TARGET=wasi CPU_TARGET=wasm32 INSTALL_PREFIX=$(pwd)/../fpcwasm
cd ..

cd ../..
{
    echo "set(CMAKE_Pascal_COMPILER \"$(pwd)/mods/langExamplePascal/fpcwasm/lib/fpc/$FPC_VERSION/ppcrosswasm32\")"
    echo "set(CMAKE_Pascal_COMPILER \"$(pwd)/mods/langExamplePascal/fpcwasm/lib/fpc/$FPC_VERSION/ppcrosswasm32\" CACHE FILEPATH \"Path to WASM compatable FreePascal compiler\")"

    echo "set(WEBROGUE_PASCAL_TOOLCHAIN_UNITS \"$(pwd)/mods/langExamplePascal/fpcwasm/lib/fpc/$FPC_VERSION/units/wasm32-wasi\")"
    echo "set(WEBROGUE_PASCAL_TOOLCHAIN_UNITS \"$(pwd)/mods/langExamplePascal/fpcwasm/lib/fpc/$FPC_VERSION/units/wasm32-wasi\" CACHE PATH \"???\")"

    echo "set(CMAKE_Pascal_COMPILER_WORKS TRUE)"
} > mods/langExamplePascal/PascalToolchainInfo.cmake
