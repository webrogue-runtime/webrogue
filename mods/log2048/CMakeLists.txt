set (CMAKE_CXX_STANDARD 17)

add_library(log2048 ${WEBROGUE_MOD_LIB_TYPE} log2048.cpp sqlpp_schema.h)

target_link_libraries(log2048 sqlpp11_sqlite3)

include(../tools/sqlpp_scheme_generator.cmake)

make_sqlpp_schema(
    DDL schema.sql
    HEADER sqlpp_schema
    NAMESPACE webrogue::log2048
)
register_mod(
    NAME log2048
)
