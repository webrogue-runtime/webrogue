set(WASI_SDK_PREFIX ${CMAKE_CURRENT_LIST_DIR}/wasi-sdk-21.0)
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
	set(WASM_SDK_URL https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-21/wasi-sdk-21.0-linux.tar.gz)
	set(WASM_SDK_SHA512 "4675d7bd90601d95a0fb0ce4c72d39bf2b9c2e9c2edef86bcc56b94169213665ea4901103f358d87db916ab7ffcc8dfb4686b3a106bec4c748b6ab83e1d7b5b9")
elseif(APPLE)
	set(WASM_SDK_URL https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-21/wasi-sdk-21.0-macos.tar.gz)
	set(WASM_SDK_SHA512 "67e86dc9fda2244e495dade8fa837fd0d8b2c76fb881a27e189953d588df09c0927e72b8d955d87588cea80becb9d27adceb525edf866b08844e48fc8bb90035")
elseif(WIN32)
	set(WASM_SDK_URL https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-21/wasi-sdk-21.0.m-mingw.tar.gz)
	set(WASM_SDK_SHA512 "bce856d89401df87c1334dc7a5e1083e1bcecb44c9510b2986f1642f35d69472893b1f9f538333fdd67c5cb360fe9a8882d1dec11dea784fa3af1773659706e9")
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
set(WEBROGUE_SYSROOT ${CMAKE_CURRENT_LIST_DIR}/wasi-sysroot)
string(APPEND TOOLCHAIN_DATA "set(WEBROGUE_SYSROOT ${WEBROGUE_SYSROOT})\n")
set(target_triple wasm32-wasi)
string(APPEND TOOLCHAIN_DATA "set(target_triple ${target_triple})\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_C_FLAGS \"--sysroot=${WEBROGUE_SYSROOT} -nostdlib -O3 -Wl,--no-entry,-r,--export-if-defined=__wasm_call_ctors\")\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_CXX_FLAGS \"--sysroot=${WEBROGUE_SYSROOT} -nostdlib -O3 -Wl,--no-entry,-r,--export-if-defined=__wasm_call_ctors\")\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_SHARED_LIBRARY_C_FLAGS \"-Wl,--no-merge-data-segments\")\n")
string(APPEND TOOLCHAIN_DATA "set(CMAKE_SHARED_LIBRARY_CXX_FLAGS \"-Wl,--no-merge-data-segments\")\n")
# string(APPEND TOOLCHAIN_DATA "set(CMAKE_LINKER=${WASI_SDK_PREFIX}/bin/wasm-ld${WASI_HOST_EXE_SUFFIX})\n")
# string(APPEND TOOLCHAIN_DATA "set(CMAKE_C_LINK_EXECUTABLE=\"${WASI_SDK_PREFIX} <FLAGS> <CMAKE_C_LINK_FLAGS> <LINK_FLAGS> <OBJECTS> -o <TARGET> <LINK_LIBRARIES>\")\n")
# string(APPEND TOOLCHAIN_DATA "set(CMAKE_CXX_LINK_EXECUTABLE=\"${WASI_SDK_PREFIX}/bin/wasm-ld${WASI_HOST_EXE_SUFFIX} <FLAGS> <CMAKE_CXX_LINK_FLAGS> <LINK_FLAGS> <OBJECTS> -o <TARGET> <LINK_LIBRARIES>\")\n")


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
