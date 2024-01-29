# Copyright (C) 2019 Intel Corporation.  All rights reserved.
# SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

cmake_minimum_required (VERSION 3.14)

include(CheckPIESupported)

function(make_vmlib)
    exec_program(git ${WEBROGUE_ROOT_PATH}/external/wamr ARGS apply ../wamr.patch OUTPUT_VARIABLE v)
    set(options NO_JIT DISABLE_HW_BOUND_CHECK)
    set(oneValueArgs PLATFORM PROCESSOR)
    set(multiValueArgs)
    cmake_parse_arguments(MAKE_VMLIB "${options}" "${oneValueArgs}" "${multiValueArgs}" ${ARGN})

    set(WAMR_BUILD_PLATFORM ${MAKE_VMLIB_PLATFORM})

    set(CMAKE_C_STANDARD 99)
    set(CMAKE_CXX_STANDARD 17)

    # Set WAMR_BUILD_TARGET, currently values supported:
    # "X86_64", "AMD_64", "X86_32", "AARCH64[sub]", "ARM[sub]", "THUMB[sub]",
    # "MIPS", "XTENSA", "RISCV64[sub]", "RISCV32[sub]"
    if(DEFINED MAKE_VMLIB_PROCESSOR)
        set(WAMR_BUILD_TARGET ${MAKE_VMLIB_PROCESSOR})
    else()
        if(CMAKE_SYSTEM_PROCESSOR MATCHES "^(arm64|aarch64)")
            set(WAMR_BUILD_TARGET "AARCH64")
        elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "riscv64")
            set(WAMR_BUILD_TARGET "RISCV64")
        elseif(CMAKE_SIZEOF_VOID_P EQUAL 8)
            set(WAMR_BUILD_TARGET "X86_64")
        elseif (CMAKE_SIZEOF_VOID_P EQUAL 4)
            set(WAMR_BUILD_TARGET "X86_32")
        else()
            message(FATAL_ERROR "Unsupported build target platform!")
        endif()
    endif()

    if(WAMR_BUILD_TARGET STREQUAL "X86_64")
        set(WAMR_HAS_JIT TRUE)
    else()
        set(WAMR_HAS_JIT FALSE)
    endif()
    if(MAKE_VMLIB_NO_JIT)
        set(WAMR_HAS_JIT FALSE)
    endif()
    set(WAMR_HAS_JIT ${WAMR_HAS_JIT} PARENT_SCOPE)

    set(WAMR_BUILD_INTERP 1)
    set(WAMR_BUILD_AOT 0)
    set(WAMR_BUILD_JIT 0)
    if(WAMR_HAS_JIT)
        set(WAMR_BUILD_FAST_JIT 1)
    else()
        set(WAMR_BUILD_FAST_JIT 0)
    endif()

    set(WAMR_BUILD_LIBC_BUILTIN 0)

    set(WAMR_BUILD_LIBC_WASI 0)

    if(WAMR_HAS_JIT)
        set(WAMR_BUILD_FAST_INTERP 0)
    else()
        set(WAMR_BUILD_FAST_INTERP 1)
    endif()

    set(WAMR_BUILD_MULTI_MODULE 0)
    set(WAMR_BUILD_LIB_PTHREAD 0)
    set(WAMR_BUILD_LIB_WASI_THREADS 0)
    set(WAMR_BUILD_MINI_LOADER 0)
    set(WAMR_BUILD_SIMD 1)
    set(WAMR_BUILD_REF_TYPES 0)
    set(WAMR_BUILD_DEBUG_INTERP 0)

    if(WAMR_BUILD_DEBUG_INTERP EQUAL 1)
        set(WAMR_BUILD_FAST_INTERP 0)
        set(WAMR_BUILD_MINI_LOADER 0)
        set(WAMR_BUILD_SIMD 0)
    endif()

    set(WAMR_ROOT_DIR ${WEBROGUE_ROOT_PATH}/external/wamr)

    include(${WAMR_ROOT_DIR}/build-scripts/runtime_lib.cmake)

    check_pie_supported()
    add_library(vmlib STATIC ${WAMR_RUNTIME_LIB_SOURCE})
    set_target_properties(vmlib PROPERTIES POSITION_INDEPENDENT_CODE ON)

    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -Wl,--gc-sections")

    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Wextra -Wformat -Wformat-security -Wshadow")
    # set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wconversion -Wsign-conversion")

    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -Wformat -Wformat-security -Wno-unused")

    if(WAMR_BUILD_TARGET MATCHES "X86_.*" OR WAMR_BUILD_TARGET STREQUAL "AMD_64")
        if(NOT (CMAKE_C_COMPILER MATCHES ".*clang.*" OR CMAKE_C_COMPILER_ID MATCHES ".*Clang"))
            set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mindirect-branch-register")
            set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -mindirect-branch-register")
            # UNDEFINED BEHAVIOR, refer to https://en.cppreference.com/w/cpp/language/ub
        endif()
    endif()


    if(MAKE_VMLIB_DISABLE_HW_BOUND_CHECK)
        target_compile_definitions(vmlib PUBLIC WASM_DISABLE_HW_BOUND_CHECK=1)
    endif()
    # The following flags are to enhance security, but it may impact performance,
    # we disable them by default.
    #if (WAMR_BUILD_TARGET MATCHES "X86_.*" OR WAMR_BUILD_TARGET STREQUAL "AMD_64")
    #  set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -ftrapv -D_FORTIFY_SOURCE=2")
    #endif ()
    #set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fstack-protector-strong --param ssp-buffer-size=4")
    #set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wl,-z,noexecstack,-z,relro,-z,now")
endfunction()
