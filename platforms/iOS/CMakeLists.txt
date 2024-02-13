cmake_minimum_required(VERSION 3.13)

set(CMAKE_OSX_DEPLOYMENT_TARGET 12.0)

project(webrogue_ios C CXX Swift)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 14)

set(CLANG_HAS_WASM FALSE)
set(WEBROGUE_ROOT_PATH ${CMAKE_CURRENT_LIST_DIR}/../..)
set(WEBROGUE_MOD_NAMES core log2048)
include(../../make_webrogue.cmake)

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


make_webrogue_output(
    LIB_NAME webrogue_output_sdl
    TYPE SDL
    STATIC
)
target_include_directories(
    webrogue_output_sdl PUBLIC 
    ${WEBROGUE_ROOT_PATH}/external/SDL/include 
    ${WEBROGUE_ROOT_PATH}/external/SDL_ttf
)
target_link_libraries(
    webrogue_output_sdl 
    SDL2d SDL2maind SDL2_ttfd freetyped
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
    LIB_NAME webrogue_runtime_m3
    TYPE WASM3
    STATIC
    NO_DEFAULT_FACRTORY
)
make_webrogue_runtime(
    LIB_NAME webrogue_runtime_wasmer
    TYPE WASM_C_API
    STATIC
    NO_DEFAULT_FACRTORY
)
target_include_directories(webrogue_runtime_wasmer PRIVATE ${WEBROGUE_ROOT_PATH}/external/wasmer/lib/c-api)

add_library(core_wrmod STATIC
    ${WEBROGUE_ROOT_PATH}/embedded_resources/core_wrmod.c ${WEBROGUE_ROOT_PATH}/embedded_resources/core_wrmod.h
)


file(GLOB_RECURSE WEBROGUE_SWIFT_SOURCES ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/*.swift)
file(GLOB_RECURSE SNAPKIT_SWIFT_SOURCES ${WEBROGUE_ROOT_PATH}/platforms/iOS/external/SnapKit/Sources/*.swift)
# add_library(SnapKit SHARED
#     ${SNAPKIT_SWIFT_SOURCES}
# )
# set_target_properties(SnapKit PROPERTIES
#   FRAMEWORK TRUE
#   FRAMEWORK_VERSION Swift
#   MACOSX_FRAMEWORK_IDENTIFIER com.cmake.dynamicFramework
#   MACOSX_FRAMEWORK_INFO_PLIST "${WEBROGUE_ROOT_PATH}/platforms/iOS/external/SnapKit/Supporting Files/Info.plist"
#   # "current version" in semantic format in Mach-O binary file
#   VERSION 16.4.0
#   # "compatibility version" in semantic format in Mach-O binary file
#   SOVERSION 1.0.0
#   XCODE_ATTRIBUTE_CODE_SIGN_IDENTITY "iPhone Developer"
# )

add_executable(
    webrogue 
    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/main.h
    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/main.mm

    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/make_m3_runtime.h
    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/make_m3_runtime.mm
    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/make_wasm_c_api_runtime.h
    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/make_wasm_c_api_runtime.mm

    ${WEBROGUE_ROOT_PATH}/platforms/iOS/webrogue-Bridging-Header.h
    ${WEBROGUE_ROOT_PATH}/external/SDL/src/video/uikit/SDL_uikitappdelegate.h

    ${WEBROGUE_SWIFT_SOURCES}

    ${SNAPKIT_SWIFT_SOURCES}
)

set_target_properties(webrogue PROPERTIES XCODE_ATTRIBUTE_LIBRARY_SEARCH_PATHS "$(SRCROOT)/cmake_build/sdl/$(CONFIGURATION)$(EFFECTIVE_PLATFORM_NAME)\n$(SRCROOT)/cmake_build/sdl_ttf/$(CONFIGURATION)$(EFFECTIVE_PLATFORM_NAME)\n$(SRCROOT)/cmake_build/sdl_ttf/external/freetype/$(CONFIGURATION)$(EFFECTIVE_PLATFORM_NAME)")
set_target_properties(webrogue PROPERTIES XCODE_ATTRIBUTE_SWIFT_OBJC_BRIDGING_HEADER "$(SRCROOT)/webrogue-Bridging-Header.h")
set(webrogue PROPERTIES CMAKE_XCODE_ATTRIBUTE_INFOPLIST_KEY_UILaunchStoryboardName "LaunchScreen.storyboard")


target_link_libraries(
    webrogue
    webrogue_core
    webrogue_runtime_wasmer
    ${WEBROGUE_ROOT_PATH}/platforms/iOS/rust_target/universal/libwasmer.a
    webrogue_runtime_m3
    webrogue_output_sdl
    core_wrmod 

    "-framework AVFoundation"
    "-framework AudioToolbox"
    "-framework CoreAudio"
    "-framework CoreBluetooth"
    "-framework CoreFoundation"
    "-framework CoreGraphics"
    "-framework CoreHaptics"
    "-framework CoreMotion"
    "-framework CoreVideo"
    "-framework GameController"
    "-framework Metal"
    "-framework OpenGLES"
    "-framework QuartzCore"
    "-framework UIKit"

    # "-framework SystemConfiguration"
    "-framework JavaScriptCore"

    "iconv"
)

add_custom_target("cmake_configure" COMMAND sh ${CMAKE_CURRENT_SOURCE_DIR}/configure.sh)

include("${WEBROGUE_ROOT_PATH}/cmake/xcode_source_grouping.cmake")

source_group(
    TREE "${WEBROGUE_ROOT_PATH}/platforms/iOS"
    FILES 
    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/main.h
    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/main.mm

    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/make_m3_runtime.h
    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/make_m3_runtime.mm
    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/make_wasm_c_api_runtime.h
    ${WEBROGUE_ROOT_PATH}/platforms/iOS/Webrogue/make_wasm_c_api_runtime.mm
    ${WEBROGUE_SWIFT_SOURCES}
)

source_group(
    TREE "${WEBROGUE_ROOT_PATH}/platforms/iOS/external"
    FILES
    ${SNAPKIT_SWIFT_SOURCES}
)

# target_source_group(TARGET webrogue ROOT_DIR ${WEBROGUE_ROOT_PATH}/platforms/iOS)

set_target_properties(webrogue PROPERTIES
    BUNDLE True
    MACOSX_BUNDLE_GUI_IDENTIFIER io.github.webrogue_runtime
    MACOSX_BUNDLE_BUNDLE_NAME webrogue
    MACOSX_BUNDLE_BUNDLE_VERSION "0.1"
    MACOSX_BUNDLE_SHORT_VERSION_STRING "0.1"
    MACOSX_BUNDLE_INFO_PLIST ${WEBROGUE_ROOT_PATH}/platforms/iOS/info.plist.in
)

target_sources(webrogue PUBLIC ${WEBROGUE_ROOT_PATH}/mods/log2048/log2048.wrmod)
set_source_files_properties(${WEBROGUE_ROOT_PATH}/mods/log2048/log2048.wrmod PROPERTIES MACOSX_PACKAGE_LOCATION Resources)


target_sources(webrogue PUBLIC ${WEBROGUE_ROOT_PATH}/platforms/iOS/LaunchScreen.storyboard)
set_source_files_properties(${WEBROGUE_ROOT_PATH}/platforms/iOS/LaunchScreen.storyboard PROPERTIES MACOSX_PACKAGE_LOCATION Resources)

target_sources(webrogue PUBLIC ${WEBROGUE_ROOT_PATH}/platforms/iOS/colors.xcassets)
set_source_files_properties(${WEBROGUE_ROOT_PATH}/platforms/iOS/colors.xcassets PROPERTIES MACOSX_PACKAGE_LOCATION Resources)
