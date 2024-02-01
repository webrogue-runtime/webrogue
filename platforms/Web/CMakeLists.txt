cmake_minimum_required(VERSION 3.13)

project(webrogue_game)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 14)

set(CLANG_HAS_WASM FALSE)
set(WEBROGUE_ROOT_PATH ${CMAKE_CURRENT_LIST_DIR}/../..)
set(WEBROGUE_MOD_NAMES core log2048)

include(${WEBROGUE_ROOT_PATH}/make_webrogue.cmake)

add_compile_options(
    -sUSE_SDL=2 
    -sUSE_SDL_TTF=2 
    -fno-rtti
)
add_link_options(
    -sUSE_SDL=2 
    -sUSE_SDL_TTF=2 
    -sASYNCIFY 
    -sALLOW_MEMORY_GROWTH 
    -sASYNCIFY_STACK_SIZE=65535
    -sFORCE_FILESYSTEM 
    -fno-rtti 
)

add_custom_command(
    OUTPUT ${CMAKE_CURRENT_LIST_DIR}/main.cpp
    COMMAND ${CMAKE_COMMAND} -E touch_nocreate ${CMAKE_CURRENT_LIST_DIR}/game/main.cpp
    DEPENDS ${CMAKE_CURRENT_LIST_DIR}/game/vmlib.js
)

make_webrogue_output(
    LIB_NAME webrogue_output_sdl
    TYPE SDL
    STATIC
)
make_webrogue_core(
    LIB_NAME webrogue_core
    STATIC
    NO_WASM
)

#libuv
set(LIBUV_BUILD_SHARED OFF)
exec_program(git ${WEBROGUE_ROOT_PATH}/external/libuv ARGS apply ${WEBROGUE_ROOT_PATH}/platforms/Web/libuv.patch OUTPUT_VARIABLE v)
add_subdirectory(${WEBROGUE_ROOT_PATH}/external/libuv libuv)
target_link_libraries(webrogue_core uv_a)
target_include_directories(webrogue_core PRIVATE ${WEBROGUE_ROOT_PATH}/external/libuv/include) 

target_link_libraries(webrogue_output_sdl webrogue_core)
make_webrogue_runtime(
    LIB_NAME webrogue_runtime_web
    TYPE WEB
    STATIC
)

set(WEBROGUE_COMPACT_LINKER_STATIC ON)
include(${WEBROGUE_ROOT_PATH}/cmake/make_compact_linker.cmake)

embed_resource(${WEBROGUE_ROOT_PATH}/mods/core/core.wrmod)
add_executable(
    webrogue_game 
    "game/main.cpp" 
    "${WEBROGUE_ROOT_PATH}/platforms/Web/uv_emscripten.c" 
    "${WEBROGUE_ROOT_PATH}/embedded_resources/core_wrmod.h"
    "${WEBROGUE_ROOT_PATH}/embedded_resources/core_wrmod.c"
)
target_include_directories(webrogue_game PRIVATE ${WEBROGUE_ROOT_PATH}/external/libuv/include) 
target_link_libraries(webrogue_game webrogue_output_sdl webrogue_core webrogue_runtime_web compact_linker idbfs.js)
target_include_directories(webrogue_game PRIVATE ${WEBROGUE_ROOT_PATH}/embedded_resources)
target_link_options(
    webrogue_game PRIVATE
    -sWASM_BIGINT
    -sFETCH 
    --js-library ${CMAKE_CURRENT_LIST_DIR}/game/vmlib.js 
    "-sASYNCIFY_IMPORTS=[presentModSelector, prepareMods, initWasmModule, execFunc, continueFuncExecution, readModMem, writeModMem]"
)

make_webrogue_runtime(
    LIB_NAME webrogue_runtime_native
    TYPE NATIVE
    STATIC
    NATIVE_RUNTIME_MODS core log2048
    WEBROGUE_CORE_LIB webrogue_core
)
add_executable(webrogue_baked_game_asmjs "baked_game/main.cpp" "${WEBROGUE_ROOT_PATH}/platforms/Web/uv_emscripten.c")
target_link_options(webrogue_baked_game_asmjs PRIVATE -sWASM=0)
target_include_directories(webrogue_baked_game_asmjs PRIVATE ${WEBROGUE_ROOT_PATH}/external/libuv/include) 
target_link_libraries(webrogue_baked_game_asmjs webrogue_output_sdl webrogue_core webrogue_runtime_native idbfs.js)
target_include_directories(webrogue_baked_game_asmjs PRIVATE ${WEBROGUE_ROOT_PATH}/embedded_resources)


add_executable(webrogue_baked_game_wasm "baked_game/main.cpp" "${WEBROGUE_ROOT_PATH}/platforms/Web/uv_emscripten.c")
target_link_options(webrogue_baked_game_wasm PRIVATE -sWASM=1)
target_include_directories(webrogue_baked_game_wasm PRIVATE ${WEBROGUE_ROOT_PATH}/external/libuv/include) 
target_link_libraries(webrogue_baked_game_wasm webrogue_output_sdl webrogue_core webrogue_runtime_native idbfs.js)
target_include_directories(webrogue_baked_game_wasm PRIVATE ${WEBROGUE_ROOT_PATH}/embedded_resources)

if(WEBROGUE_ASMJS)
    set(WEBROGUE_SECOUND_FILE_EXTENSION "")
else()
    set(WEBROGUE_SECOUND_FILE_EXTENSION "")
endif()

set(ARTIFACTS_PATH ${CMAKE_CURRENT_LIST_DIR})

if(CMAKE_BUILD_TYPE STREQUAL Debug)
    set(WEBROGUE_COPY_JS_MEM_COMMAND)
else()
    set(WEBROGUE_COPY_JS_MEM_COMMAND COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/webrogue_baked_game_asmjs.js.mem ${ARTIFACTS_PATH}/webrogue_baked_game_asmjs.js.mem)
endif()

add_custom_target(pack_artifacts
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/webrogue_game.js ${ARTIFACTS_PATH}/webrogue_game.js
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/webrogue_game.wasm ${ARTIFACTS_PATH}/webrogue_game.wasm
    COMMAND ${CMAKE_COMMAND} -E make_directory ${ARTIFACTS_PATH}/mods
    COMMAND ${CMAKE_COMMAND} -E copy ${WEBROGUE_ROOT_PATH}/mods/log2048/log2048.wrmod ${ARTIFACTS_PATH}/mods/log2048.wrmod

    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/webrogue_baked_game_asmjs.js ${ARTIFACTS_PATH}/webrogue_baked_game_asmjs.js
    # ${WEBROGUE_COPY_JS_MEM_COMMAND}

    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/webrogue_baked_game_wasm.js ${ARTIFACTS_PATH}/webrogue_baked_game_wasm.js
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/webrogue_baked_game_wasm.wasm ${ARTIFACTS_PATH}/webrogue_baked_game_wasm.wasm
    
    DEPENDS webrogue_baked_game_asmjs
    DEPENDS webrogue_baked_game_wasm
    DEPENDS webrogue_game 
    DEPENDS ${WEBROGUE_ROOT_PATH}/mods/log2048/log2048.wrmod
)
