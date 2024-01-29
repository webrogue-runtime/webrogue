file(
    DOWNLOAD "https://github.com/WasmEdge/llvm-windows/releases/download/llvmorg-16.0.6/LLVM-16.0.6-win64-MultiThreadedDLL.zip" ${CMAKE_BINARY_DIR}/LLVM-16.0.6-win64-MultiThreadedDLL.zip
    EXPECTED_MD5 "1773796e6c428e435189ed403a652d02"
    SHOW_PROGRESS
)

execute_process(COMMAND ${CMAKE_COMMAND} -E tar xfz ${CMAKE_BINARY_DIR}/LLVM-16.0.6-win64-MultiThreadedDLL.zip
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    RESULT_VARIABLE rv)
if(NOT rv EQUAL 0)
    message(FATAL_ERROR "error: extraction of '${_filename}' failed")
endif()

set(LLVM_DIR ${CMAKE_BINARY_DIR}/LLVM-16.0.6-win64/lib/cmake/llvm)
