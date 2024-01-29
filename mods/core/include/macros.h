#pragma once

#if !defined(WEBROGUE_NATIVE_MODS) && !defined(WEBROGUE_WASM_MODS)
#error missing webrogue mod defenitions
#endif

#if !defined(WEBROGUE_NATIVE_MODS)
#define WR_IMPORTED(RETURN_TYPE, NAME)                                         \
    __attribute__((import_name(#NAME)))                                        \
    __attribute__((import_module("webrogue"))) RETURN_TYPE NAME
#else
#define WR_IMPORTED(RETURN_TYPE, NAME) RETURN_TYPE NAME
#endif

#if !defined(WEBROGUE_NATIVE_MODS)
#define WR_EXPORTED(RETURN_TYPE, NAME)                                         \
    __attribute__((export_name(#NAME))) RETURN_TYPE NAME
#else
#define WR_EXPORTED(RETURN_TYPE, NAME) RETURN_TYPE NAME
#endif
