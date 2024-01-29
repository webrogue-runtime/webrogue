set (CMAKE_CXX_STANDARD 17)

add_library(
    core ${WEBROGUE_MOD_LIB_TYPE} 
    src/core.cpp src/wrsqlite.cpp 
    src/exceptions.cpp 
    src/wrout.cpp
)

#sqlpp11
set(BUILD_SQLITE3_CONNECTOR ON)
add_subdirectory(../../external/wrsqlpp11 sqlpp11)
target_link_libraries(core sqlpp11::sqlite3)

register_mod(
    NAME core
)
