cmake_minimum_required(VERSION 3.22.1)

project("webrogue_common")

set(WEBROGUE_SDL_VERSION "2.30.10")
set(WEBROGUE_SDL_HASH "d5ebf6989428ad54a1769fe91be7811ff2ab33ac638fdab29b5d04b88a39dff1")

set(WEBROGUE_SDL_DIR_NAME "SDL-release-${WEBROGUE_SDL_VERSION}")
set(WEBROGUE_SDL_DIR "${CMAKE_CURRENT_LIST_DIR}/external/${WEBROGUE_SDL_DIR_NAME}")

# file(
#     DOWNLOAD
#    	"https://codeload.github.com/libsdl-org/SDL/zip/refs/tags/release-${WEBROGUE_SDL_VERSION}"
#     "${WEBROGUE_SDL_DIR}.zip"
# 	EXPECTED_HASH "SHA256=${WEBROGUE_SDL_HASH}"
# 	SHOW_PROGRESS
# )
# if(NOT EXISTS "${WEBROGUE_SDL_DIR}")
#     file(
#         ARCHIVE_EXTRACT
#         INPUT "${WEBROGUE_SDL_DIR}.zip"
#         DESTINATION "${CMAKE_CURRENT_LIST_DIR}/external"
#     )
# endif()

set(SDL_SHARED ON)
set(SDL_STATIC OFF)
add_subdirectory("${WEBROGUE_SDL_DIR}" SDL2)
# set(SDL2TTF_VENDORED ON)
# set(SDL2TTF_INSTALL OFF)
# add_subdirectory(external/SDL_ttf)

set(WEBROGUE_GFXSTREAM_DIR ${CMAKE_CURRENT_LIST_DIR}/../../../../../../crates/gfxstream)
include(${WEBROGUE_GFXSTREAM_DIR}/paths.cmake)

add_library(
    webrogue_static STATIC
    ../../../../../../crates/gfx-fallback/webrogue_gfx_ffi_sdl2.c
    ../../../../../../crates/gfx-fallback/webrogue_gfx_ffi_sdl2_events.c
    ${WEBROGUE_GFXSTREAM_SOURCES}
)

add_library(
    webrogue SHARED
    webrogue_runtime.c
)

target_link_libraries(webrogue webrogue_static)
target_link_libraries(webrogue_static SDL2)



target_include_directories(
    webrogue_static PRIVATE
    ${WEBROGUE_GFXSTREAM_INCLUDE_DIRS}
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

target_link_libraries(
    webrogue

    android
    log
    ${RUST_TARGET_DIR}/${CARGO_TARGET}/${CARGO_PROFILE}/libwebrogue_android.a
    SDL2
    # SDL2_ttf
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
