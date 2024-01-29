file(DOWNLOAD
   	"https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-21/wasi-sysroot-21.0.tar.gz"
    ${CMAKE_CURRENT_LIST_DIR}/wasi-sysroot.tar.gz
	EXPECTED_HASH SHA512=9009dd6fa95746d82f0c22b0ebc8f35db928afa178b9f4f643ac2afe169237c1c4a440077716788ba876bae1b61ab29a1399e02e15115b1eb72ce9f6c48c2ecf
	SHOW_PROGRESS
)
file(ARCHIVE_EXTRACT INPUT ${CMAKE_CURRENT_LIST_DIR}/wasi-sysroot.tar.gz DESTINATION ${CMAKE_CURRENT_LIST_DIR})
