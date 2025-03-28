cmake_minimum_required(VERSION 3.23)

project(webrogue)

exec_program(git "${CMAKE_CURRENT_LIST_DIR}/external" ARGS clone https://github.com/libsdl-org/SDL.git OUTPUT_VARIABLE ignored)
exec_program(git "${CMAKE_CURRENT_LIST_DIR}/external/SDL" ARGS apply ../sdl3.patch OUTPUT_VARIABLE ignored)

set(SDL_SHARED OFF)
set(SDL_STATIC ON)

add_compile_options(
    -pthread
    -sSUPPORT_LONGJMP=0
)
add_subdirectory("${CMAKE_CURRENT_LIST_DIR}/external/SDL" SDL3)
add_subdirectory("${CMAKE_CURRENT_LIST_DIR}/../crates/gfxstream" gfxstream)

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_PROFILE_FLAG)
    set(CARGO_PROFILE_DIR debug)
    set(ASYNCIFY_FLAG -sJSPI=1)
else()
    set(CARGO_PROFILE_FLAG --release)
    set(CARGO_PROFILE_DIR release)
    set(ASYNCIFY_FLAG -sASYNCIFY=1)
endif()


include(ExternalProject)
ExternalProject_Add(
    webrogue_web
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo +nightly build --manifest-path=../crates/web/Cargo.toml "--target-dir=${CMAKE_SOURCE_DIR}/target" -Zbuild-std=std,panic_abort --target=wasm32-unknown-emscripten ${CARGO_PROFILE_FLAG}
    BINARY_DIR "${CMAKE_SOURCE_DIR}"
    SOURCE_DIR
    INSTALL_COMMAND ""  
    BUILD_ALWAYS ON
)

set(WRAPP_PATH "${CMAKE_SOURCE_DIR}/../examples/raylib/raylib.wrapp")

add_link_options(
    ${ASYNCIFY_FLAG}
    -sFORCE_FILESYSTEM 
    -fno-rtti
    -sWASMFS
    -lopfs.js
    -sOFFSCREEN_FRAMEBUFFER
    -pthread
    -sMIN_WEBGL_VERSION=2
    # -sFULL_ES3
    # -sGL_DEBUG
    # -sGL_ASSERTIONS
    # -sTRACE_WEBGL_CALLS
    --preload-file ${WRAPP_PATH}@main.wrapp
    -sALLOW_MEMORY_GROWTH
    -sPTHREAD_POOL_SIZE=1
    -sEXPORTED_FUNCTIONS=_main,_wr_rs_exported_fn,_wr_rs_exported_async_fn
    -sWASM_BIGINT
    -sSTACK_SIZE=2097152
    # "-sASYNCIFY_EXPORTS=[wr_rs_exported_async_fn]"
    -sPROXY_TO_PTHREAD
    -sSUPPORT_LONGJMP=0
)

add_executable(
    webrogue
    main.c
    ../crates/web/web.c
    ../crates/gfx-fallback/webrogue_gfx_ffi_sdl3.c
    ../crates/gfx-fallback/webrogue_gfx_ffi_sdl3_events.c
)

add_custom_target(example_wrapp DEPENDS ${WRAPP_PATH})

add_dependencies(webrogue webrogue_web example_wrapp)
target_link_libraries(
    webrogue 
    "${CMAKE_SOURCE_DIR}/target/wasm32-unknown-emscripten/${CARGO_PROFILE_DIR}/libwebrogue_web.a"
    SDL3-static
    webrogue_gfxstream
)
