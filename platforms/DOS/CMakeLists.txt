cmake_minimum_required(VERSION 3.13)

project(webrogue)

set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 14)

set(CLANG_HAS_WASM FALSE)
set(WEBROGUE_ROOT_PATH ${CMAKE_CURRENT_LIST_DIR}/../..)


add_compile_definitions(SQLITE_THREADSAFE=0)
add_compile_definitions(SQLITE_OMIT_WAL)
add_compile_definitions(ETIMEDOUT=789)
add_compile_options(-Wno-attributes)
set(WEBROGUE_MOD_NAMES core log2048)

include(../../make_webrogue.cmake)

make_webrogue_output(
    LIB_NAME webrogue_output_pdcurses
    TYPE PDCURSES
    STATIC
    PDCURSES_OS dos
)

make_webrogue_core(
    LIB_NAME webrogue_core
    STATIC 
    NO_WASM
    NO_WASI
)
make_webrogue_runtime(
    LIB_NAME webrogue_runtime_native
    TYPE NATIVE
    STATIC
    NATIVE_RUNTIME_MODS log2048 core
    WEBROGUE_CORE_LIB webrogue_core
)
target_link_libraries(log2048 core)

target_link_libraries(webrogue_runtime_native webrogue_core)

add_executable(webrogue "./main.cpp")
target_include_directories(webrogue PRIVATE ${WEBROGUE_ROOT_PATH}/embedded_resources)

target_include_directories(webrogue PRIVATE ${WEBROGUE_ROOT_PATH}/src)
target_link_libraries(
    webrogue
    webrogue_core
    webrogue_runtime_native
    webrogue_output_pdcurses
)

add_custom_target(make_artifacts_dir
    COMMAND ${CMAKE_COMMAND} -E rm -rf ${WEBROGUE_ROOT_PATH}/artifacts
    COMMAND ${CMAKE_COMMAND} -E make_directory ${WEBROGUE_ROOT_PATH}/artifacts
)

add_custom_target(pack_executable_to_artifacts
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_BINARY_DIR}/webrogue ${WEBROGUE_ROOT_PATH}/artifacts/WEBROGUE.EXE
    COMMAND ${CMAKE_COMMAND} -E copy ${WEBROGUE_ROOT_PATH}/platforms/DOS/CWSDPMI.EXE ${WEBROGUE_ROOT_PATH}/artifacts/

    DEPENDS webrogue make_artifacts_dir
)
