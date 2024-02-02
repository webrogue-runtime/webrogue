set (CMAKE_C_STANDARD 99)
set (CMAKE_CXX_STANDARD 14)

add_library(
    langExampleCore ${WEBROGUE_MOD_LIB_TYPE}

    langExampleCore.cpp
)

register_mod(
    NAME langExampleCore
)
