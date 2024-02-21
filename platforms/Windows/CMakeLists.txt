#cmake_minimum_required(VERSION 3.13)

#project(webrogue-pc)

#set(Platform "Windows" CACHE INTERNAL "" FORCE)
#add_subdirectory(../ build-windows)

#add_executable(webrogue_windows ${PROJECT_SOURCE_DIR}/win.cpp)
#target_link_libraries(webrogue_windows webrogue_sdl)

cmake_minimum_required(VERSION 3.13)

project(webrogue C CXX)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 14)

set(CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS 1)

set(CLANG_HAS_WASM FALSE)
set(WEBROGUE_ROOT_PATH ${CMAKE_CURRENT_LIST_DIR}/../..)
set(WEBROGUE_MOD_NAMES core log2048)

include(../../make_webrogue.cmake)

include_directories(${WEBROGUE_ROOT_PATH}/external/dirent/include)

include(${WEBROGUE_ROOT_PATH}/platforms/Windows/download_sdl.cmake)
add_compile_definitions("MAKE_DEFAULT_RUNTIME_EXPORT=__declspec(dllexport)")

make_webrogue_core(
    LIB_NAME webrogue_core
    SHARED
)
set(WEBROGUE_COMPACT_LINKER_SHARED ON)
include(${WEBROGUE_ROOT_PATH}/cmake/make_compact_linker.cmake)
target_link_libraries(webrogue_core compact_linker)

#libuv
set(LIBUV_BUILD_SHARED ON)
add_subdirectory(${WEBROGUE_ROOT_PATH}/external/libuv libuv)
target_link_libraries(webrogue_core uv)
target_include_directories(webrogue_core PRIVATE ${WEBROGUE_ROOT_PATH}/external/libuv/include) 

make_webrogue_runtime(
    LIB_NAME webrogue_runtime_wasmer
    TYPE WASM_C_API
    SHARED
)

link_to_wasmer(
    LIB_NAME webrogue_runtime_wasmer
    SHARED
)

target_link_libraries(webrogue_runtime_wasmer webrogue_core)

make_webrogue_output(
    LIB_NAME webrogue_output_sdl
    TYPE SDL
    STATIC
)
target_link_libraries(webrogue_output_sdl ${SDL2_LIBRARY} ${SDL2_TTF_LIBRARY} webrogue_core)
target_include_directories(webrogue_output_sdl PRIVATE ${SDL2_INCLUDE_DIR} ${SDL2_TTF_INCLUDE_DIR})

make_webrogue_output(
    LIB_NAME webrogue_output_pdcurses
    TYPE PDCURSES
    STATIC
    PDCURSES_OS wincon
)

add_executable(webrogue ${CMAKE_CURRENT_LIST_DIR}/win.cpp)
target_link_libraries(webrogue webrogue_core webrogue_output_pdcurses webrogue_runtime_wasmer)
target_include_directories(webrogue PRIVATE ${WEBROGUE_ROOT_PATH}/src)

add_executable(webrogue_sdl WIN32 ${CMAKE_CURRENT_LIST_DIR}/win_sdl.cpp)
target_link_libraries(webrogue_sdl webrogue_core webrogue_output_sdl webrogue_runtime_wasmer)
target_include_directories(webrogue_sdl PRIVATE ${WEBROGUE_ROOT_PATH}/src ${SDL2_INCLUDE_DIR} ${SDL2_TTF_INCLUDE_DIR})

add_custom_command(
    TARGET webrogue_sdl
    POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy ${SDL2_DLLS} ${SDL2_TTF_DLLS} $<PATH:GET_PARENT_PATH,$<TARGET_FILE:webrogue_sdl>>
)
add_custom_target(webrogue_windows DEPENDS webrogue webrogue_sdl pack_mods)
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} /STACK:8388608")

set(ARTIFACTS_PATH ${WEBROGUE_ROOT_PATH}/artifacts)

add_custom_target(pack_artifacts 
    COMMAND ${CMAKE_COMMAND} -E rm -rf ${ARTIFACTS_PATH}
    COMMAND ${CMAKE_COMMAND} -E make_directory ${ARTIFACTS_PATH}
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:webrogue> $<TARGET_FILE:webrogue_core> $<TARGET_FILE:webrogue_runtime_wasmer> $<TARGET_FILE:webrogue_sdl> $<TARGET_FILE:uv> ${SDL2_DLLS} ${SDL2_TTF_DLLS} $<TARGET_FILE:compact_linker> ${ARTIFACTS_PATH}
    COMMAND ${CMAKE_COMMAND} -E make_directory ${ARTIFACTS_PATH}/mods
    COMMAND ${CMAKE_COMMAND} -E copy ${WEBROGUE_ROOT_PATH}/mods/core/core.wrmod ${WEBROGUE_ROOT_PATH}/mods/log2048/log2048.wrmod ${ARTIFACTS_PATH}/mods
    
    DEPENDS webrogue_windows
)
