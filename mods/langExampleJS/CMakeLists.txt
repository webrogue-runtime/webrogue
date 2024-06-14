set (CMAKE_CXX_STANDARD 17)
set (CMAKE_C_STANDARD 99)

exec_program(git ${CMAKE_CURRENT_LIST_DIR}/external/quickjs ARGS apply ../quickjs.patch OUTPUT_VARIABLE v)

add_library(
    langExampleJS ${WEBROGUE_MOD_LIB_TYPE}

    langExampleJS.c

    external/quickjs/quickjs.c
    external/quickjs/libunicode.c
    external/quickjs/libbf.c
    external/quickjs/cutils.c
    external/quickjs/libregexp.c
)

target_compile_definitions(
    langExampleJS
    PRIVATE
    -DCONFIG_VERSION="2024-02-14"
    -DWEBROGUE
    -DUSE_BF_DEC
)


register_mod(
    NAME langExampleJS
)
