cmake_minimum_required(VERSION 3.23)

project(webrogue)

exec_program(git "${CMAKE_CURRENT_LIST_DIR}/external" ARGS clone https://github.com/libsdl-org/SDL.git OUTPUT_VARIABLE ignored)
exec_program(git "${CMAKE_CURRENT_LIST_DIR}/external/SDL" ARGS apply ../sdl3.patch OUTPUT_VARIABLE ignored)

set(SDL_SHARED OFF)
set(SDL_STATIC ON)

add_compile_options(
    # -pthread
    -sSUPPORT_LONGJMP=0
)
add_subdirectory("${CMAKE_CURRENT_LIST_DIR}/external/SDL" SDL3)

include(ExternalProject)
ExternalProject_Add(
    webrogue_web
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo +nightly build -Zbuild-std=std,panic_abort --target=wasm32-unknown-emscripten --release
    BINARY_DIR "${CMAKE_SOURCE_DIR}/crates/web"
    SOURCE_DIR
    INSTALL_COMMAND ""  
    BUILD_ALWAYS ON
)


add_link_options(
    # -sASYNCIFYJSPI
    # -sASYNCIFY_STACK_SIZE=65535
    -sJSPI=1
    -sFORCE_FILESYSTEM 
    -fno-rtti
    -sWASMFS
    -lopfs.js
    -sOFFSCREEN_FRAMEBUFFER
    # -sOFFSCREENCANVAS_SUPPORT=1
    # -pthread
    -sMIN_WEBGL_VERSION=2
    --embed-file ${CMAKE_SOURCE_DIR}/../examples/raylib/raylib.wrapp@raylib.wrapp
    # -sPTHREAD_POOL_SIZE=1
    -sEXPORTED_FUNCTIONS=_main,_wr_rs_exported_fn,_wr_rs_exported_async_fn
    -sWASM_BIGINT
    # "-sASYNCIFY_IMPORTS=[wr_em_js_initWasmModule]"
    "-sJSPI_IMPORTS=[wr_em_js_initWasmModule, wr_em_js_execFunc]"
    "-sJSPI_EXPORTS=[wr_rs_exported_async_fn]"
    # -sPROXY_TO_PTHREAD
    -sSUPPORT_LONGJMP=0
)

add_executable(
    webrogue
    main.c
    crates/backend-web/src/em_js.c
    ../crates/gfx-ffi/src/webrogue_gfx_ffi_sdl3.c
)

add_custom_target(example_wrapp DEPENDS ${CMAKE_SOURCE_DIR}/../examples/raylib/raylib.wrapp)

add_dependencies(webrogue webrogue_web example_wrapp)
target_link_libraries(
    webrogue 
    "${CMAKE_SOURCE_DIR}/crates/web/target/wasm32-unknown-emscripten/release/libwebrogue_web.a"
    SDL3-static
)
