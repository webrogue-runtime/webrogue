set (CMAKE_C_STANDARD 99)

add_library(
    langExampleC ${WEBROGUE_MOD_LIB_TYPE}

    langExampleC.c
)

register_mod(
    NAME langExampleC
)
