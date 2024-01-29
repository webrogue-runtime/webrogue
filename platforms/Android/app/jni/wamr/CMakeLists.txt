project(wamr C CXX ASM)
include(${WEBROGUE_ROOT_PATH}/cmake/make_vmlib.cmake)

if(${ANDROID_ABI} STREQUAL "x86_64")
    set(WAMR_HAS_JIT FALSE)
    set(WAMR_BUILD_TARGET "X86_64")
elseif(${ANDROID_ABI} STREQUAL "x86")
    set(WAMR_HAS_JIT FALSE)
    set(WAMR_BUILD_TARGET "X86_32")
elseif(${ANDROID_ABI} STREQUAL "arm64-v8a")
    set(WAMR_HAS_JIT FALSE)
    set(WAMR_BUILD_TARGET "AARCH64")
elseif(${ANDROID_ABI} STREQUAL "armeabi-v7a")
    set(WAMR_HAS_JIT FALSE)
    set(WAMR_BUILD_TARGET "ARM")
else()
    message(FATAL_ERROR "Unknown ANDROID_ABI: ${ANDROID_ABI}")
endif()

make_vmlib(
    PLATFORM android
    PROCESSOR ${WAMR_BUILD_TARGET}
    NO_JIT
    DISABLE_HW_BOUND_CHECK
)
