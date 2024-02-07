set (CMAKE_C_STANDARD 99)

add_library(
    resExample ${WEBROGUE_MOD_LIB_TYPE}

    resExample.c
)

register_mod(
    NAME resExample
)
