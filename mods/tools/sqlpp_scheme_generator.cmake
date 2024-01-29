
function(make_sqlpp_schema)
    set(options)
    set(oneValueArgs DDL HEADER NAMESPACE)
    set(multiValueArgs)
    cmake_parse_arguments(ARGS "${options}" "${oneValueArgs}" "${multiValueArgs}" ${ARGN})
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/${ARGS_HEADER}.h
        COMMAND python3 ${CMAKE_CURRENT_SOURCE_DIR}/../../external/wrsqlpp11/scripts/ddl2cpp ${CMAKE_CURRENT_SOURCE_DIR}/${ARGS_DDL} ${CMAKE_CURRENT_SOURCE_DIR}/${ARGS_HEADER} ${ARGS_NAMESPACE}
        DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/${ARGS_DDL} ${CMAKE_CURRENT_SOURCE_DIR}/../../external/wrsqlpp11/scripts/ddl2cpp
    )
endfunction()
