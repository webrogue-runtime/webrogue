cmake_minimum_required(VERSION 3.22.1)

project("webrogue_common")

set(WEBROGUE_SDL_VERSION "3.2.10")
set(WEBROGUE_SDL_DIR_NAME "SDL-release-${WEBROGUE_SDL_VERSION}")
set(WEBROGUE_SDL_DIR "${CMAKE_CURRENT_LIST_DIR}/external/${WEBROGUE_SDL_DIR_NAME}")

if (CMAKE_BUILD_TYPE STREQUAL "Release" OR CMAKE_BUILD_TYPE STREQUAL "RelWithDebInfo")
    add_link_options(-Wl,--strip-debug -Wl,--gc-sections)
endif()

set(SDL_SHARED ON)
set(SDL_STATIC OFF)
add_subdirectory("${WEBROGUE_SDL_DIR}" SDL3)
# set(SDL2TTF_VENDORED ON)
# set(SDL2TTF_INSTALL OFF)
# add_subdirectory(external/SDL_ttf)

set(WEBROGUE_GFXSTREAM_DIR ${CMAKE_CURRENT_LIST_DIR}/../../../../../../crates/gfxstream)
include(${WEBROGUE_GFXSTREAM_DIR}/paths.cmake)

add_library(
    webrogue_static STATIC
    ../../../../../../crates/gfx-fallback/webrogue_gfx_ffi_sdl.c
    ../../../../../../crates/gfx-fallback/webrogue_gfx_ffi_sdl_events.c
    ${WEBROGUE_GFXSTREAM_SOURCES}
)

target_compile_definitions(webrogue_static PRIVATE WEBROGUE_GFX_SDL_VERSION=3)

add_library(
    webrogue SHARED
    webrogue_runtime.c
)

target_link_libraries(webrogue webrogue_static)
target_link_libraries(webrogue_static SDL3::SDL3-shared)


target_include_directories(
    webrogue PRIVATE
    "${CMAKE_CURRENT_LIST_DIR}/external/SDL-release-${WEBROGUE_SDL_VERSION}/include"
)
target_include_directories(
    webrogue_static PRIVATE
    ${WEBROGUE_GFXSTREAM_INCLUDE_DIRS}
    "${CMAKE_CURRENT_LIST_DIR}/external/SDL-release-${WEBROGUE_SDL_VERSION}/include"
)

if(CMAKE_SYSTEM_PROCESSOR MATCHES "^(arm64|aarch64)")
    set(CARGO_TARGET "aarch64-linux-android")
elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv7-a")
    set(CARGO_TARGET "armv7-linux-androideabi")
elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "i686")
    set(CARGO_TARGET "i686-linux-android")
elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "x86_64")
    set(CARGO_TARGET "x86_64-linux-android")
else()
    message(FATAL_ERROR "Unknown CMAKE_SYSTEM_PROCESSOR: ${CMAKE_SYSTEM_PROCESSOR}")
endif()

if(CARGO_PROFILE STREQUAL "dev")
    set(CARGO_PROFILE "debug")
endif()


target_link_libraries(
    webrogue

    android
    log
    ${RUST_TARGET_DIR}/${CARGO_TARGET}/${CARGO_PROFILE}/libwebrogue_android.a
    SDL3::SDL3-shared
)

# message(FATAL_ERROR "${WEBROGUE_SDL_DIR}/android-project/app/src/main/java/org/libsdl/app/" 
#     "${CMAKE_CURRENT_SOURCE_DIR}/../java/org/libsdl/app")
    
execute_process(
    COMMAND ${CMAKE_COMMAND} -E make_directory 
    "${CMAKE_CURRENT_SOURCE_DIR}/../java/org/libsdl"
)
execute_process(
    COMMAND ${CMAKE_COMMAND} -E copy_directory 
    "${WEBROGUE_SDL_DIR}/android-project/app/src/main/java/org/libsdl/app/" 
    "${CMAKE_CURRENT_SOURCE_DIR}/../java/org/libsdl/app"
)
