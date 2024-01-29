cmake_minimum_required(VERSION 3.13)


project(webrogue_baked_game)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 14)

set(CLANG_HAS_WASM FALSE)
set(WEBROGUE_ROOT_PATH ${CMAKE_CURRENT_LIST_DIR}/../../..)
set(WEBROGUE_MOD_NAMES core log2048)

include(../../../make_webrogue.cmake)

add_compile_options(-sUSE_SDL=2 -sUSE_SDL_TTF=2 -fno-rtti)
add_link_options(-sUSE_SDL=2 -sUSE_SDL_TTF=2 -sASYNCIFY -sALLOW_MEMORY_GROWTH -sASYNCIFY_STACK_SIZE=65535 -sFORCE_FILESYSTEM -fno-rtti)


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
    LIB_NAME webrogue_runtime_native
    TYPE NATIVE
    STATIC
    NATIVE_RUNTIME_MODS core log2048
    WEBROGUE_CORE_LIB webrogue_core
)
add_executable(webrogue_baked_game_asmjs "./main.cpp" "../uv_emscripten.c")
target_link_options(webrogue_baked_game_asmjs PRIVATE -sWASM=0)
target_include_directories(webrogue_baked_game_asmjs PRIVATE ${WEBROGUE_ROOT_PATH}/external/libuv/include) 
target_link_libraries(webrogue_baked_game_asmjs webrogue_output_sdl webrogue_core webrogue_runtime_native idbfs.js)
target_include_directories(webrogue_baked_game_asmjs PRIVATE ${WEBROGUE_ROOT_PATH}/embedded_resources)


add_executable(webrogue_baked_game_wasm "./main.cpp" "../uv_emscripten.c")
target_link_options(webrogue_baked_game_wasm PRIVATE -sWASM=1)
target_include_directories(webrogue_baked_game_wasm PRIVATE ${WEBROGUE_ROOT_PATH}/external/libuv/include) 
target_link_libraries(webrogue_baked_game_wasm webrogue_output_sdl webrogue_core webrogue_runtime_native idbfs.js)
target_include_directories(webrogue_baked_game_wasm PRIVATE ${WEBROGUE_ROOT_PATH}/embedded_resources)

if(WEBROGUE_ASMJS)
    set(WEBROGUE_SECOUND_FILE_EXTENSION "")
else()
    set(WEBROGUE_SECOUND_FILE_EXTENSION "")
endif()

if(CMAKE_BUILD_TYPE STREQUAL Debug)
    set(WEBROGUE_COPY_JS_MEM_COMMAND)
else()
    set(WEBROGUE_COPY_JS_MEM_COMMAND COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/webrogue_baked_game_asmjs.js.mem ${WEBROGUE_ROOT_PATH}/artifacts/webrogue_baked_game_asmjs.js.mem)
endif()

add_custom_target(pack_artifacts
    COMMAND ${CMAKE_COMMAND} -E rm -rf ${WEBROGUE_ROOT_PATH}/artifacts
    COMMAND ${CMAKE_COMMAND} -E make_directory ${WEBROGUE_ROOT_PATH}/artifacts
    
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/webrogue_baked_game_asmjs.js ${WEBROGUE_ROOT_PATH}/artifacts/webrogue_baked_game_asmjs.js
    ${WEBROGUE_COPY_JS_MEM_COMMAND}

    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/webrogue_baked_game_wasm.js ${WEBROGUE_ROOT_PATH}/artifacts/webrogue_baked_game_wasm.js
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/webrogue_baked_game_wasm.wasm ${WEBROGUE_ROOT_PATH}/artifacts/webrogue_baked_game_wasm.wasm
    
    DEPENDS webrogue_baked_game_asmjs webrogue_baked_game_wasm
)
