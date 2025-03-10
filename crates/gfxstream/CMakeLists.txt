project(WEBROGUE_GFXSTREAM CXX)
set (CMAKE_CXX_STANDARD 17)

set(WEBROGUE_GFXSTREAM_DIR ${CMAKE_CURRENT_LIST_DIR})
include(${WEBROGUE_GFXSTREAM_DIR}/paths.cmake)

add_library(
    webrogue_gfxstream STATIC
    ${WEBROGUE_GFXSTREAM_SOURCES}
)

target_include_directories(
    webrogue_gfxstream PRIVATE
    ${WEBROGUE_GFXSTREAM_INCLUDE_DIRS}
)
set(CMAKE_INSTALL_LIBDIR "")
install(TARGETS webrogue_gfxstream)
