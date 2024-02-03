set -ex
#-DCMAKE_VERBOSE_MAKEFILE=ON 

cmake -B build/mods_build -S mods --toolchain=tools/generated_toolchain.cmake -DCMAKE_MODULE_PATH=../cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_LINKER=a "-DWEBROGUE_MOD_NAMES=core;langExampleCore;langExampleC;langExamplePascal" -DWEBROGUE_PASCAL_TOOLCHAIN_COMPILER=$HOME/fpcwasm/lib/fpc/3.3.1/ppcrosswasm32 -DWEBROGUE_PASCAL_TOOLCHAIN_UNITS=$HOME/fpcwasm/lib/fpc/3.3.1/units/wasm32-wasi

cmake --build build/mods_build --target final_linking