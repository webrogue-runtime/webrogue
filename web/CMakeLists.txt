cmake_minimum_required(VERSION 3.23)

project(webrogue)

exec_program(git "${CMAKE_CURRENT_LIST_DIR}/external" ARGS clone https://github.com/libsdl-org/SDL.git OUTPUT_VARIABLE ignored)
exec_program(git "${CMAKE_CURRENT_LIST_DIR}/external/SDL" ARGS checkout 3a2b595c4f93366413e1c7c82e6e99218ead8803 OUTPUT_VARIABLE ignored)
exec_program(git "${CMAKE_CURRENT_LIST_DIR}/external/SDL" ARGS apply ../sdl3.patch OUTPUT_VARIABLE ignored)

set(SDL_SHARED OFF)
set(SDL_STATIC ON)

set(RUSTFLAGS "-Ctarget-feature=+atomics,+bulk-memory,+mutable-globals --codegen panic=abort")

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_PROFILE_FLAG)
    set(CARGO_PROFILE_DIR debug)
    set(ASYNCIFY_FLAG -sJSPI=1)
    set(FS_FLAG WEBROGUE_JSPI=1)
else()
    set(CARGO_PROFILE_FLAG --release)
    set(CARGO_PROFILE_DIR release)
    set(ASYNCIFY_FLAG -sASYNCIFY=1)
    set(RUSTFLAGS "${RUSTFLAGS} -Clinker-plugin-lto")
    set(FS_FLAG WEBROGUE_JSPI=0)
    add_compile_options(-flto)
    add_link_options(-flto)
endif()

add_compile_options(
    -pthread
    -sSUPPORT_LONGJMP=0
)
add_subdirectory("${CMAKE_CURRENT_LIST_DIR}/external/SDL" SDL3)
add_subdirectory("${CMAKE_CURRENT_LIST_DIR}/../crates/gfxstream" gfxstream)

include(ExternalProject)
ExternalProject_Add(
    webrogue_web
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND ${CMAKE_COMMAND} -E env "RUSTFLAGS=${RUSTFLAGS}" 
        cargo +nightly build --manifest-path=../crates/web/Cargo.toml 
        "--target-dir=${CMAKE_SOURCE_DIR}/target" 
        -Zbuild-std=std,panic_abort 
        --target=wasm32-unknown-emscripten 
        ${CARGO_PROFILE_FLAG}
    BINARY_DIR "${CMAKE_CURRENT_SOURCE_DIR}"
    SOURCE_DIR
    INSTALL_COMMAND ""  
    BUILD_ALWAYS ON
)

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
    -sALLOW_MEMORY_GROWTH
    -sPTHREAD_POOL_SIZE=1
    -sEXPORTED_FUNCTIONS=_main,_wr_rs_exported_fn,_wr_rs_exported_async_fn,_wr_allocApp
    -sEXPORTED_RUNTIME_METHODS=callMain
    -sWASM_BIGINT
    -sSTACK_SIZE=2097152
    -sPROXY_TO_PTHREAD
    -sSUPPORT_LONGJMP=0
)

if(WEBROGUE_WEB_DYNAMIC)
    add_link_options(-sINVOKE_RUN=0)
else()
    set(WRAPP_PATH "${CMAKE_SOURCE_DIR}/../examples/raylib/raylib.wrapp")
    add_custom_target(example_wrapp DEPENDS ${WRAPP_PATH})
    add_link_options(--preload-file ${WRAPP_PATH}@main.wrapp)
endif()

add_executable(
    webrogue
    main.c
    ../crates/web/web.c
    ../crates/gfx-fallback/webrogue_gfx_ffi_sdl.c
    ../crates/gfx-fallback/webrogue_gfx_ffi_sdl_events.c
)
add_dependencies(webrogue webrogue_web)

target_compile_definitions(webrogue PRIVATE ${FS_FLAG} WEBROGUE_GFX_SDL_VERSION=3)

if(WEBROGUE_WEB_DYNAMIC)
    target_compile_definitions(webrogue PRIVATE WEBROGUE_DYNAMIC=1)
else()
    target_compile_definitions(webrogue PRIVATE WEBROGUE_DYNAMIC=0)
    add_dependencies(webrogue example_wrapp)
endif()

target_link_libraries(
    webrogue 
    "${CMAKE_SOURCE_DIR}/target/wasm32-unknown-emscripten/${CARGO_PROFILE_DIR}/libwebrogue_web.a"
    SDL3-static
    webrogue_gfxstream
)
