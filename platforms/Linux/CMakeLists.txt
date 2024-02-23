cmake_minimum_required(VERSION 3.13)

project(webrogue)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 14)

execute_process(
    COMMAND uname -o
    OUTPUT_VARIABLE linux_os
)
string(STRIP ${linux_os} linux_os)
if(${linux_os} STREQUAL "Android")
  set(WEBROGUE_TERMUX ON)
else()
  set(WEBROGUE_TERMUX OFF)
endif()


option(WEBROGUE_USE_SDL "Build webrogue with sdl support" ON)

set(CLANG_HAS_WASM FALSE)
set(WEBROGUE_ROOT_PATH ${CMAKE_CURRENT_LIST_DIR}/../..)
set(WEBROGUE_MOD_NAMES core log2048)

include(../../make_webrogue.cmake)

make_webrogue_output(
    LIB_NAME webrogue_output_ncurses
    TYPE NCURSES
    STATIC
)
make_webrogue_core(
    LIB_NAME webrogue_core
    STATIC
)
target_link_libraries(webrogue_core compact_linker)

#libuv
set(LIBUV_BUILD_SHARED OFF)
add_subdirectory(${WEBROGUE_ROOT_PATH}/external/libuv libuv EXCLUDE_FROM_ALL)
target_link_libraries(webrogue_core uv_a)
target_include_directories(webrogue_core PRIVATE ${WEBROGUE_ROOT_PATH}/external/libuv/include)


set(CURSES_NEED_NCURSES true)
find_package(Curses REQUIRED)
include_directories(${CURSES_INCLUDE_DIR})
target_link_libraries(webrogue_output_ncurses ${CURSES_LIBRARIES})

set(BUILD_SHARED_LIBS OFF)
if(WEBROGUE_TERMUX)
    set(WEBROGUE_SAVED_VAR_ANDROID ${ANDROID})
    unset(ANDROID)
endif()


if(WEBROGUE_TERMUX)
    set(ANDROID ${WEBROGUE_SAVED_VAR_ANDROID})
endif()

make_webrogue_runtime(
    LIB_NAME webrogue_runtime_wasmtime
    TYPE WASM_C_API
    STATIC
)
target_link_libraries(webrogue_runtime_wasmtime webrogue_core)
link_to_wasmer(
    LIB_NAME webrogue_runtime_wasmtime
    STATIC
)
list(APPEND WEBROGUE_OUTPUTS ncurses)
if(${WEBROGUE_USE_SDL})
    # if(WEBROGUE_APPIMAGE)
    set(SDL_SHARED OFF)
    set(SDL_STATIC ON)
    add_subdirectory(${WEBROGUE_ROOT_PATH}/external/SDL SDL2)
    set(SDL2_INCLUDE_DIR ${WEBROGUE_ROOT_PATH}/external/SDL/include)
    set(SDL2_LIBRARY SDL2-static)

    set(SDL2TTF_VENDORED ON)
    set(SDL2TTF_INSTALL OFF)
    add_subdirectory(${WEBROGUE_ROOT_PATH}/external/SDL_ttf SDL_ttf)
    set(SDL2_TTF_INCLUDE_DIR ${WEBROGUE_ROOT_PATH}/external/SDL_ttf)
    set(SDL2_TTF_LIBRARY SDL2_ttf)
    # else()
    #     list(APPEND CMAKE_MODULE_PATH ${WEBROGUE_ROOT_PATH}/cmake/sdl2)
    #     find_package(SDL2)
    #     find_package(SDL2_ttf)
    # endif()
    make_webrogue_output(
        LIB_NAME webrogue_output_sdl
        TYPE SDL
        STATIC
    )
    add_executable(webrogue main_with_sdl.cpp find_data_path.cpp)
    target_include_directories(webrogue_output_sdl PRIVATE ${SDL2_INCLUDE_DIR} ${SDL2_TTF_INCLUDE_DIR})
    target_include_directories(webrogue PRIVATE ${SDL2_INCLUDE_DIR} ${SDL2_TTF_INCLUDE_DIR})
    target_link_libraries(webrogue_output_sdl ${SDL2_LIBRARY} ${SDL2_TTF_LIBRARY})
    target_link_libraries(webrogue webrogue_output_sdl)
else()
    add_executable(webrogue main.cpp find_data_path.cpp)
endif()

add_library(core_wrmod STATIC
    ${WEBROGUE_ROOT_PATH}/embedded_resources/core_wrmod.c ${WEBROGUE_ROOT_PATH}/embedded_resources/core_wrmod.h
)

target_include_directories(webrogue PRIVATE ${WEBROGUE_ROOT_PATH}/src)

set(WEBROGUE_COMPACT_LINKER_STATIC ON)
include(${WEBROGUE_ROOT_PATH}/cmake/make_compact_linker.cmake)

target_link_libraries(
    webrogue 
    core_wrmod 
    webrogue_output_ncurses
    webrogue_core
    webrogue_runtime_wasmtime
)

if(WEBROGUE_TERMUX)
    target_link_libraries(webrogue android-posix-semaphore)
endif()

target_compile_features(webrogue PRIVATE cxx_std_17)

if(WEBROGUE_APPIMAGE)
    install(
        CODE 
        "include(${WEBROGUE_ROOT_PATH}/platforms/Linux/appimage.cmake)
        make_appimage(
            EXE \"${CMAKE_BINARY_DIR}/webrogue\"
            NAME \"YourAppName\"
            ICON \"${WEBROGUE_ROOT_PATH}/platforms/Linux/512.png\"
            DIR_ICON \"${WEBROGUE_ROOT_PATH}/platforms/Linux/512.png\"
            OUTPUT_NAME \"${WEBROGUE_ROOT_PATH}/platforms/Linux/App.AppImage\"
            ASSETS \"${CMAKE_BINARY_DIR}/mods\"
            SHARED_LIBS \"/usr/lib/x86_64-linux-gnu/libncurses.so.6\" \"/usr/lib/x86_64-linux-gnu/libtinfo.so.6\"
        )
        " 
        COMPONENT Runtime
    )
    target_link_libraries(webrogue -static-libstdc++)
    target_compile_options(webrogue BEFORE PRIVATE -static)
else()
    install(TARGETS webrogue)
    install(
        FILES ${WEBROGUE_ROOT_PATH}/mods/log2048/log2048.wrmod
        DESTINATION share/webrogue/mods
    )
    set(CPACK_GENERATOR "DEB")
    set(CPACK_DEBIAN_PACKAGE_MAINTAINER "Artem Borovik")
    set(CPACK_DEBIAN_PACKAGE_DEPENDS "libc6 (>= 2.14), libgcc-s1 (>= 10), libstdc++6 (>= 12), libncursesw6 (>= 6)")
    include(CPack)
endif()
# libsdl2-2.0-0 (>= 2), libsdl2-ttf-2.0-0 (>= 2), 
