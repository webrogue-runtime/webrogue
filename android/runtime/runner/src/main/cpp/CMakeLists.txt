cmake_minimum_required(VERSION 3.22.1)

project("webrogue_runner")

set(RUST_TARGET_DIR ${CMAKE_CURRENT_LIST_DIR}/../rust_target)
add_subdirectory("${CMAKE_CURRENT_LIST_DIR}/../../../../common/src/main/cpp" webrogue_runner_common)
target_link_libraries(
    webrogue 
    "${CMAKE_CURRENT_LIST_DIR}/aot.o"
)
