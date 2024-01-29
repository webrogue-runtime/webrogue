set(WASI_SDK_PREFIX ${CMAKE_CURRENT_LIST_DIR}/wasi-sdk-20.0)
if(WIN32)
	set(WASI_SDK_PREFIX "${WASI_SDK_PREFIX}+m")
endif()
set(NEEDED_PROGRAMS 
	${WASI_SDK_PREFIX}/bin/clang${WASI_HOST_EXE_SUFFIX}
	${WASI_SDK_PREFIX}/bin/clang++${WASI_HOST_EXE_SUFFIX}
	${WASI_SDK_PREFIX}/bin/clang${WASI_HOST_EXE_SUFFIX}
	${WASI_SDK_PREFIX}/bin/llvm-ar${WASI_HOST_EXE_SUFFIX}
	${WASI_SDK_PREFIX}/bin/llvm-ranlib${WASI_HOST_EXE_SUFFIX}
)
foreach(NEEDED_PROGRAM ${NEEDED_PROGRAMS})
	if(NOT EXISTS ${NEEDED_PROGRAM})
		set(HAS_NEEDED_PROGRAMS FALSE)
	endif()
endforeach()
if(UNIX AND NOT APPLE)
	set(WASM_SDK_URL https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-20/wasi-sdk-20.0-linux.tar.gz)
	set(WASM_SDK_SHA512 "ff3d368267526887534f50767ff010bd368e9c24178ab2f0cf57a8ed0b3a82fbf85986d620ab2327ac6bb3f456c65adc6edb80626a1289e630dde7e43b191b42")
elseif(APPLE)
	set(WASM_SDK_URL https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-20/wasi-sdk-20.0-macos.tar.gz)
	set(WASM_SDK_SHA512 "76ce659c01efe2beeee3edc082ca50794739e069fb6367e0b1ac91cb64ca0162122692afb95d598b9e38928d49753399af183d1c855bc79a6fc79171c011e66b")
elseif(WIN32)
	set(WASM_SDK_URL https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-20/wasi-sdk-20.0.m-mingw.tar.gz)
	set(WASM_SDK_SHA512 "1931a714806c0c98b94584dda7f5ecd0a49a9be886bd858221dcb34cfb7b10bb7492e5702d7b30a1583cae83610f109ed1099226c6b4f7967dacd55b73b662c6")
endif()

if(NOT DEFINED WASM_SDK_URL)
	message(FATAL_ERROR "Could not download wasi-sdk automaticaly for this platform. Install it manualy.")
endif()
file(DOWNLOAD
   	${WASM_SDK_URL}
    ${CMAKE_CURRENT_LIST_DIR}/wasi-sdk.tar.gz
	EXPECTED_HASH SHA512=${WASM_SDK_SHA512}
	SHOW_PROGRESS
)
file(ARCHIVE_EXTRACT INPUT ${CMAKE_CURRENT_LIST_DIR}/wasi-sdk.tar.gz DESTINATION ${CMAKE_CURRENT_LIST_DIR})

if(WIN32)
	set(WASI_HOST_EXE_SUFFIX ".exe")
else()
	set(WASI_HOST_EXE_SUFFIX "")
endif()

set(TOOLCHAIN_DATA "")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_SYSTEM_NAME wasi)\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_SYSTEM_PROCESSOR wasm)\n")
set(WEBROGUE_SYSROOT ${CMAKE_CURRENT_LIST_DIR}/wasi-sdk-20.0/share/wasi-sysroot)
string(APPEND TOOLCHAIN_DATA "set(WEBROGUE_SYSROOT ${WEBROGUE_SYSROOT})\n")
set(target_triple wasm32-wasi)
string(APPEND TOOLCHAIN_DATA "set(target_triple ${target_triple})\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_C_FLAGS \"--sysroot=${WEBROGUE_SYSROOT} -nostdlib -O3 -Wl,--no-entry,-r,--export-if-defined=__wasm_call_ctors\")\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_CXX_FLAGS \"--sysroot=${WEBROGUE_SYSROOT} -nostdlib -O3 -Wl,--no-entry,-r,--export-if-defined=__wasm_call_ctors\")\n")

string(APPEND TOOLCHAIN_DATA "set(CMAKE_C_COMPILER ${WASI_SDK_PREFIX}/bin/clang${WASI_HOST_EXE_SUFFIX})\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_CXX_COMPILER ${WASI_SDK_PREFIX}/bin/clang++${WASI_HOST_EXE_SUFFIX})\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_ASM_COMPILER ${WASI_SDK_PREFIX}/bin/clang${WASI_HOST_EXE_SUFFIX})\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_AR ${WASI_SDK_PREFIX}/bin/llvm-ar${WASI_HOST_EXE_SUFFIX})\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_RANLIB ${WASI_SDK_PREFIX}/bin/llvm-ranlib${WASI_HOST_EXE_SUFFIX})\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_C_COMPILER_TARGET ${target_triple})\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_CXX_COMPILER_TARGET ${target_triple})\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_ASM_COMPILER_TARGET ${target_triple})\n")

string(APPEND TOOLCHAIN_DATA "set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)\n")

file(WRITE ${CMAKE_CURRENT_LIST_DIR}/generated_toolchain.cmake "${TOOLCHAIN_DATA}")

set(LINKER_PATH_DATA "")
string(APPEND LINKER_PATH_DATA "set(WEBROGUE_MODS_LINKER ${WASI_SDK_PREFIX}/bin/wasm-ld${WASI_HOST_EXE_SUFFIX})\n")

file(WRITE ${CMAKE_CURRENT_LIST_DIR}/linker_path.cmake "${LINKER_PATH_DATA}")
