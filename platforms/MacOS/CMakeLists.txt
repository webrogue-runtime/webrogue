cmake_minimum_required(VERSION 3.13)

set(CMAKE_OSX_DEPLOYMENT_TARGET 11.0)

project(webrogue_macos)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 14)

set(CMAKE_XCODE_ATTRIBUTE_FRAMEWORK_SEARCH_PATHS "$(SRCROOT)")
set(CMAKE_XCODE_ATTRIBUTE_ARCHS "$(ARCHS_STANDARD)")

set(CLANG_HAS_WASM TRUE)
set(WEBROGUE_ROOT_PATH ${CMAKE_CURRENT_LIST_DIR}/../..)
set(WEBROGUE_MOD_NAMES core log2048)
include(../../make_webrogue.cmake)

make_webrogue_output(
    LIB_NAME webrogue_output_sdl
    TYPE SDL
    STATIC
)
set(SDL_STATIC ON CACHE BOOL "")
set(SDL_STATIC ON)
set(SDL_SHARED OFF CACHE BOOL "")
set(SDL_SHARED OFF)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "")
set(BUILD_SHARED_LIBS OFF)
set(SDL2TTF_VENDORED ON CACHE BOOL "")
set(SDL2TTF_VENDORED ON)
exec_program(git ${WEBROGUE_ROOT_PATH}/external/sdl ARGS apply ../sdl.patch OUTPUT_VARIABLE v)
add_subdirectory(../../external/SDL sdl)
add_subdirectory(../../external/SDL_ttf sdl_ttf)
target_include_directories(
    webrogue_output_sdl PUBLIC 
    ${WEBROGUE_ROOT_PATH}/external/SDL/include 
    ${WEBROGUE_ROOT_PATH}/external/SDL_ttf
)
target_link_libraries(
    webrogue_output_sdl 
    SDL2-static SDL2main SDL2_ttf freetype
)
make_webrogue_core(
    LIB_NAME webrogue_core
    STATIC
)

set(WEBROGUE_COMPACT_LINKER_STATIC ON)
include(${WEBROGUE_ROOT_PATH}/cmake/make_compact_linker.cmake)
target_link_libraries(webrogue_core compact_linker)

#libuv
set(LIBUV_BUILD_SHARED OFF)
add_subdirectory(${WEBROGUE_ROOT_PATH}/external/libuv libuv)
target_link_libraries(webrogue_core uv_a)
target_include_directories(webrogue_core PRIVATE ${WEBROGUE_ROOT_PATH}/external/libuv/include) 

make_webrogue_runtime(
    LIB_NAME webrogue_runtime_wasmer
    TYPE WASM_C_API
    STATIC
)
target_include_directories(webrogue_runtime_wasmer PRIVATE ${WEBROGUE_ROOT_PATH}/external/wasmer/lib/c-api)
add_library(core_wrmod STATIC
    ${WEBROGUE_ROOT_PATH}/embedded_resources/core_wrmod.c ${WEBROGUE_ROOT_PATH}/embedded_resources/core_wrmod.h
)
add_executable(webrogue MACOSX_BUNDLE ${WEBROGUE_ROOT_PATH}/platforms/MacOS/Webrogue/main.mm)

target_link_libraries(
    webrogue
    webrogue_core
    webrogue_runtime_wasmer
    ${WEBROGUE_ROOT_PATH}/platforms/MacOS/rust_target/universal/libwasmer.a
    webrogue_output_sdl
    core_wrmod
    "-framework SystemConfiguration"
    "-framework JavaScriptCore"
)

add_custom_target("cmake_configure" COMMAND sh ${CMAKE_CURRENT_SOURCE_DIR}/configure.sh)

include("${WEBROGUE_ROOT_PATH}/cmake/xcode_source_grouping.cmake")

target_source_group(TARGET webrogue ROOT_DIR "${WEBROGUE_ROOT_PATH}/src")

set_target_properties(webrogue PROPERTIES
    BUNDLE True
    MACOSX_BUNDLE_GUI_IDENTIFIER io.github.webrogue_runtime
    MACOSX_BUNDLE_BUNDLE_NAME webrogue
    MACOSX_BUNDLE_BUNDLE_VERSION "0.1"
    MACOSX_BUNDLE_SHORT_VERSION_STRING "0.1"
    XCODE_ATTRIBUTE_INFOPLIST_KEY_LSApplicationCategoryType "public.app-category.games"
    # MACOSX_BUNDLE_INFO_PLIST ${CMAKE_SOURCE_DIR}/cmake/customtemplate.plist.in
)

target_sources(webrogue PUBLIC ${WEBROGUE_ROOT_PATH}/mods/log2048/log2048.wrmod)
set_source_files_properties(${WEBROGUE_ROOT_PATH}/mods/log2048/log2048.wrmod PROPERTIES MACOSX_PACKAGE_LOCATION Resources)
