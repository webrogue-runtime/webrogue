execute_process(
    COMMAND ${WASM_OBJDUMP_PATH} -x -r -d ${INPUT_PATH}
    OUTPUT_FILE ${OUTPUT_PATH}
)