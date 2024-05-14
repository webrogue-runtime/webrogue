set(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} "${CMAKE_CURRENT_SOURCE_DIR}/cmake")

include(${CMAKE_CURRENT_LIST_DIR}/PascalToolchainInfo.cmake)

enable_language(Pascal)

add_library(
    langExamplePascal ${WEBROGUE_MOD_LIB_TYPE}

    langExamplePascal.pas
)
set_target_properties(langExamplePascal PROPERTIES LINKER_LANGUAGE C)

add_custom_command(
    TARGET langExamplePascal
    POST_BUILD
    COMMAND ${CMAKE_AR} qLs $<TARGET_FILE:langExamplePascal> ${WEBROGUE_PASCAL_TOOLCHAIN_UNITS}/*/*.o
)

register_mod(
    NAME langExamplePascal
    ALREADY_EXPORTED
)
