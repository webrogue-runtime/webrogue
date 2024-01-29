#  build/mods_build/linked.wasm -j Import

cmake_minimum_required(VERSION 3.10)

find_program(WASM_OBJDUMP wasm-objdump)

if(${WASM_OBJDUMP} STREQUAL WASM_OBJDUMP-NOTFOUND)
    message(WARNING "wasm-objdump program not found")
else()
    exec_program("${WASM_OBJDUMP}" ARGS ${LINKED_WASM_PATH} -x -j Import OUTPUT_VARIABLE OUTP)

    string(REGEX MATCHALL "- func\\[[0-9]+\\] sig=[^\n]*\n?" IMPORTS ${OUTP})
    set(ALLOWED_WASI_IMPORTS
        # wasi_snapshot_preview1.fd_close
        # wasi_snapshot_preview1.fd_write
        # wasi_snapshot_preview1.fd_seek
    )

    set(UNSUPPOERTED_IMPORTS)

    foreach(IMPORT ${IMPORTS})
        string(REGEX MATCH "<- .+$" IMPORT_NAME ${IMPORT})
        string(SUBSTRING ${IMPORT_NAME} 3 -1 IMPORT_NAME)
        string(STRIP ${IMPORT_NAME} IMPORT_NAME)
        if(${IMPORT_NAME} MATCHES "webrogue\\..+")
            continue()
        endif()
        if(IMPORT_NAME IN_LIST ALLOWED_WASI_IMPORTS)
            continue()
        endif()
        if(UNSUPPOERTED_IMPORTS)
            set(UNSUPPOERTED_IMPORTS "${UNSUPPOERTED_IMPORTS},\n\t${IMPORT_NAME}")
        else()
            set(UNSUPPOERTED_IMPORTS "\t${IMPORT_NAME}")
        endif()
    endforeach()

    if(UNSUPPOERTED_IMPORTS)
        message(FATAL_ERROR "Illegal Wasm imports: \n${UNSUPPOERTED_IMPORTS}.\n You are probably using unsupported function\nLink command is: ${TEST_LINK_COMMAND}")
    endif()
    message("Wasm imports validated")
endif()
