
set("CMAKE_SYSTEM_NAME" "Windows")
set("CMAKE_SYSTEM_PROCESSOR" "AMD64")

set(TOOLCHAIN_PREFIX "x86_64-w64-mingw32-")
set("CMAKE_C_COMPILER" "${TOOLCHAIN_PREFIX}clang")
set("CMAKE_C_COMPILER_AR" "${TOOLCHAIN_PREFIX}llvm-ar")
set("CMAKE_RC_COMPILER" "${TOOLCHAIN_PREFIX}windres")
set("CMAKE_LINKER" "ld.lld")
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)
set("CMAKE_C_LINK_EXECUTABLE" "${TOOLCHAIN_PREFIX}clang")
