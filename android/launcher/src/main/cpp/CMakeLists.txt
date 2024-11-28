cmake_minimum_required(VERSION 3.22.1)

project("webrogue_launcher")

set(RUST_TARGET_DIR ${CMAKE_CURRENT_LIST_DIR}/../rust_target)
add_subdirectory("${CMAKE_CURRENT_LIST_DIR}/../../../../common/src/main/cpp" webrogue_launcher_common)
