cmake_minimum_required(VERSION 3.13)

if(MAKE_WEBROGUE_TARGET_NAME)
    project(webrogue_mods)
else()
    project(webrogue_mods)
endif()

set(MODS_ROOT_PATH ${CMAKE_CURRENT_SOURCE_DIR})

set(WEBROGUE_MOD_DEFS WEBROGUE_WASM_MODS)

function(register_mod)
    set(options)
    set(oneValueArgs NAME)
    cmake_parse_arguments(ADD_MOD "${options}" "${oneValueArgs}" "${multiValueArgs}" ${ARGN})
    if(NOT ${WEBROGUE_MOD_NAME} STREQUAL ${ADD_MOD_NAME})
        message(FATAL_ERROR "Mod name must match directory name, got ${NAME} and ${WEBROGUE_MOD_NAME}")
    endif()
    set_target_properties(${ADD_MOD_NAME}
                      PROPERTIES SUFFIX ".wasm"
                      POSITION_INDEPENDENT_CODE OFF)

    target_link_options(${ADD_MOD_NAME} PUBLIC -Wl,--export-all,--relocatable)

    set(BUNDLED_FILE_NAME ${CMAKE_CURRENT_SOURCE_DIR}/mod.a)
    set(BUNDLING_DIR ${CMAKE_CURRENT_BINARY_DIR}/mod_bundling)


    # add_executable(${ADD_MOD_NAME}_linked ../dummy.c)
    # target_link_libraries(${ADD_MOD_NAME}_linked ${ADD_MOD_NAME})
    # target_link_options(${ADD_MOD_NAME}_linked PUBLIC -Wl,--export-all,--relocatable,--whole-archive)

    add_custom_command(
        OUTPUT ${BUNDLED_FILE_NAME}
        COMMAND ${CMAKE_COMMAND} -E rm -f ${BUNDLED_FILE_NAME}
        COMMAND ${ar_tool} qLcs ${BUNDLED_FILE_NAME} $<TARGET_FILE:${ADD_MOD_NAME}>

        # COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:${ADD_MOD_NAME}> ${BUNDLED_FILE_NAME}

        # COMMAND ${WEBROGUE_MODS_LINKER} $<TARGET_FILE:${ADD_MOD_NAME}> --trace --whole-archive --no-entry --relocatable --export-if-defined=wr_start --export=init_mod_${ADD_MOD_NAME} --allow-undefined -o ${BUNDLED_FILE_NAME}.wasm
        # COMMAND ${CMAKE_COMMAND} -E rm -f ${BUNDLED_FILE_NAME}
        # COMMAND ${ar_tool} rcs ${BUNDLED_FILE_NAME} ${BUNDLED_FILE_NAME}.wasm

        DEPENDS ${ADD_MOD_NAME}
    )

    add_custom_target(
        bundle_${ADD_MOD_NAME}
        COMMAND ${CMAKE_COMMAND} -E echo "Bundling ${ADD_MOD_NAME}"
        DEPENDS ${BUNDLED_FILE_NAME}
    )
endfunction()

add_compile_definitions(${WEBROGUE_MOD_DEFS})

set(mods ${WEBROGUE_MOD_NAMES})

set(
    WASM_LD_FLAGS 
    --export=wr_start 
    --export=__wasm_call_ctors 
    -z stack-size=5242880
    --trace
    --no-entry
    --fatal-warnings
    --no-gc-sections 
    --stack-first
    --no-merge-data-segments 
)
set(WASM_LD_DEPENDENCIES)

set(WEBROGUE_MOD_LIB_TYPE STATIC)

set(ar_tool ${CMAKE_AR})
if (CMAKE_INTERPROCEDURAL_OPTIMIZATION)
    set(ar_tool ${CMAKE_CXX_COMPILER_AR})
endif()

include(${CMAKE_CURRENT_LIST_DIR}/tools/linker_path.cmake)

foreach(mod ${mods})
    set(WEBROGUE_MOD_NAME ${mod})
    add_subdirectory(${mod})
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/${mod}/${mod}.wrmod
        COMMAND python3 ${MODS_ROOT_PATH}/tools/pack_mods.py normal ${CMAKE_CURRENT_SOURCE_DIR}/${mod}/${mod}.wrmod ${CMAKE_CURRENT_SOURCE_DIR}/${mod}
        COMMAND python3 ${MODS_ROOT_PATH}/tools/pack_mods.py nowasm ${CMAKE_CURRENT_SOURCE_DIR}/${mod}/${mod}.ewrmod ${CMAKE_CURRENT_SOURCE_DIR}/${mod}
        DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/${mod}/mod.a ${MODS_ROOT_PATH}/tools/pack_mods.py bundle_${mod}
        COMMENT "Packing ${MOD_TARGET_NAME}"
    )
    list(APPEND WASM_LD_FLAGS "${CMAKE_CURRENT_SOURCE_DIR}/${mod}/mod.a" "--export=init_mod_${mod}")
    list(APPEND WASM_LD_DEPENDENCIES ${CMAKE_CURRENT_SOURCE_DIR}/${mod}/${mod}.wrmod ${CMAKE_CURRENT_SOURCE_DIR}/${mod}/mod.a)
endforeach()

set(
    TEST_LINK_COMMAND 
    ${WEBROGUE_MODS_LINKER} ${WASM_LD_FLAGS} -o ${CMAKE_CURRENT_BINARY_DIR}/linked.wasm 

    "${CMAKE_CURRENT_SOURCE_DIR}/../mods/core/stdlibs.a"
)

# message(FATAL_ERROR ${WEBROGUE_MODS_LINKER} ${WASM_LD_FLAGS} -o ${CMAKE_CURRENT_BINARY_DIR}/linked.wasm)

add_custom_command(
    OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/linked.wasm
    COMMAND ${TEST_LINK_COMMAND}
    # COMMAND ${CMAKE_COMMAND} -DLINKED_WASM_PATH=${CMAKE_CURRENT_BINARY_DIR}/linked.wasm -DTEST_LINK_COMMAND="${TEST_LINK_COMMAND}" -P tools/validate_imports.cmake
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    DEPENDS ${WASM_LD_DEPENDENCIES}
)

set(FINAL_LINKING_DEPENDENCIES)

foreach(mod ${mods})
    list(APPEND FINAL_LINKING_DEPENDENCIES ${CMAKE_CURRENT_SOURCE_DIR}/${mod}/${mod}.wrmod)
endforeach()

add_custom_target(
    final_linking
    DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/linked.wasm ${FINAL_LINKING_DEPENDENCIES}
)
