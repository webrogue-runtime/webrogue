if(NOT EXISTS ${CMAKE_CURRENT_LIST_DIR}/generated_toolchain.cmake)
    exec_program(${CMAKE_COMMAND} ARGS -P ${CMAKE_CURRENT_LIST_DIR}/download_toolchain.cmake)
endif()
if(NOT EXISTS ${CMAKE_CURRENT_LIST_DIR}/wasi-sysroot)
    exec_program(${CMAKE_COMMAND} ARGS -P ${CMAKE_CURRENT_LIST_DIR}/download_sysroot.cmake)  
endif()

execute_process(
    COMMAND ${CMAKE_COMMAND} 
        -B ${MODS_BUILD_DIR} -S ${CMAKE_CURRENT_LIST_DIR}/..
        --toolchain=tools/generated_toolchain.cmake
        ${MODS_BUILD_GENERATOR_ARGS}
        -DCMAKE_MODULE_PATH=${CMAKE_CURRENT_LIST_DIR}/../../cmake 
        -DCMAKE_BUILD_TYPE=Release
        -DCMAKE_LINKER=a
        "-DWEBROGUE_MOD_NAMES=${WEBROGUE_MOD_NAMES}"
)
execute_process(
    COMMAND ${CMAKE_COMMAND} --build ${MODS_BUILD_DIR} --target final_linking
)
