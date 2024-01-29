project(wasm_files C CXX)

cmake_minimum_required(VERSION 3.13)

include(linker_path.cmake)

add_executable(
    t1
    t1_1.c
    t1_2.c
    t1_3.c
)

add_link_options(t1 --export=my_main)

add_custom_command(
    OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/t1_1.dump ${CMAKE_CURRENT_SOURCE_DIR}/t1_1.wasm
    COMMAND ${CMAKE_COMMAND} -DWASM_OBJDUMP_PATH=${WASM_LD_MINI_WASM_OBJDUMP_PATH} -DINPUT_PATH=${CMAKE_CURRENT_BINARY_DIR}/CMakeFiles/t1.dir/t1_1.c.obj -DOUTPUT_PATH=${CMAKE_CURRENT_SOURCE_DIR}/t1_1.dump -P ${CMAKE_CURRENT_SOURCE_DIR}/objdump_wrapper.cmake
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/CMakeFiles/t1.dir/t1_1.c.obj ${CMAKE_CURRENT_SOURCE_DIR}/t1_1.wasm
    DEPENDS t1
)

add_custom_command(
    OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/t1_2.dump ${CMAKE_CURRENT_SOURCE_DIR}/t1_2.wasm
    COMMAND ${CMAKE_COMMAND} -DWASM_OBJDUMP_PATH=${WASM_LD_MINI_WASM_OBJDUMP_PATH} -DINPUT_PATH=${CMAKE_CURRENT_BINARY_DIR}/CMakeFiles/t1.dir/t1_2.c.obj -DOUTPUT_PATH=${CMAKE_CURRENT_SOURCE_DIR}/t1_2.dump -P ${CMAKE_CURRENT_SOURCE_DIR}/objdump_wrapper.cmake
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/CMakeFiles/t1.dir/t1_2.c.obj ${CMAKE_CURRENT_SOURCE_DIR}/t1_2.wasm
    DEPENDS t1
)

add_custom_command(
    OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/t1_3.dump ${CMAKE_CURRENT_SOURCE_DIR}/t1_3.wasm
    COMMAND ${CMAKE_COMMAND} -DWASM_OBJDUMP_PATH=${WASM_LD_MINI_WASM_OBJDUMP_PATH} -DINPUT_PATH=${CMAKE_CURRENT_BINARY_DIR}/CMakeFiles/t1.dir/t1_3.c.obj -DOUTPUT_PATH=${CMAKE_CURRENT_SOURCE_DIR}/t1_3.dump -P ${CMAKE_CURRENT_SOURCE_DIR}/objdump_wrapper.cmake
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/CMakeFiles/t1.dir/t1_3.c.obj ${CMAKE_CURRENT_SOURCE_DIR}/t1_3.wasm
    
    DEPENDS t1
)

set(
    LINKER_INPUT_FILES
    ${CMAKE_CURRENT_SOURCE_DIR}/../../../mods/log2048/mod.a
    ${CMAKE_CURRENT_SOURCE_DIR}/../../../mods/core/mod.a

    ${CMAKE_CURRENT_SOURCE_DIR}/../../../mods/core/stdlibs.a
)

set(
    LINK_COMMAND

    ${WEBROGUE_MODS_LINKER} 
    ${LINKER_INPUT_FILES} 
    -o ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld.wasm 
    --no-entry 

    --export=init_mod_log2048 
    --export=init_mod_core
    --export=wr_start 
    --export=__wasm_call_ctors

    --no-gc-sections 
    --stack-first 
    --no-merge-data-segments 
    -z stack-size=700000
)

message(INFO "LINK_COMMAND=${LINK_COMMAND}")

add_custom_command(
    OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld.dump ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld.wasm
    COMMAND ${LINK_COMMAND}
    COMMAND ${CMAKE_COMMAND} -DWASM_OBJDUMP_PATH=${WASM_LD_MINI_WASM_OBJDUMP_PATH} -DINPUT_PATH=${CMAKE_CURRENT_SOURCE_DIR}/t1_lld.wasm -DOUTPUT_PATH=${CMAKE_CURRENT_SOURCE_DIR}/t1_lld.dump -P ${CMAKE_CURRENT_SOURCE_DIR}/objdump_wrapper.cmake

    DEPENDS ${LINKER_INPUT_FILES}
)

add_custom_command(
    OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.dump ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.wasm
    COMMAND ${WASM_LD_MINI_WASM_STRIP_PATH} -o ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.wasm ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld.wasm
    COMMAND ${CMAKE_COMMAND} -DWASM_OBJDUMP_PATH=${WASM_LD_MINI_WASM_OBJDUMP_PATH} -DINPUT_PATH=${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.wasm -DOUTPUT_PATH=${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.dump -P ${CMAKE_CURRENT_SOURCE_DIR}/objdump_wrapper.cmake

    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld.wasm
)

# add_custom_command(
#     OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.dump
#     COMMAND bash -c "cp ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld.wasm ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.wasm"
#     COMMAND bash -c "wasm-strip ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.wasm"
#     COMMAND bash -c "../wasm-objdump -x -r -d ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.wasm >${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.dump"
#     DEPENDS  ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld.dump
# )

add_custom_target(
    dumps ALL
    DEPENDS 
        ${CMAKE_CURRENT_SOURCE_DIR}/t1_1.dump
        ${CMAKE_CURRENT_SOURCE_DIR}/t1_1.wasm

        ${CMAKE_CURRENT_SOURCE_DIR}/t1_2.dump
        ${CMAKE_CURRENT_SOURCE_DIR}/t1_2.wasm

        ${CMAKE_CURRENT_SOURCE_DIR}/t1_3.dump
        ${CMAKE_CURRENT_SOURCE_DIR}/t1_3.wasm

        ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld.dump
        ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld.wasm

        ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.dump
        ${CMAKE_CURRENT_SOURCE_DIR}/t1_lld_strip.wasm
)