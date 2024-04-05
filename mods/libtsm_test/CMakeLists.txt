set (CMAKE_C_STANDARD 99)

add_library(
    libtsm_test ${WEBROGUE_MOD_LIB_TYPE}

    libtsm_test.cpp
)

register_mod(
    NAME libtsm_test
)
