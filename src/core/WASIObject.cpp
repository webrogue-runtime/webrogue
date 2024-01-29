#include "WASIObject.hpp"
#include "Config.hpp"
#include "ModsRuntime.hpp"
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#include "uvwasi.h"
#include "wasi_templates.hpp"
#include "wasm_types.hpp"

// #if defined(__DJGPP__)
// #include <time.h>
// typedef int clockid_t;
// static inline int clock_getres(int clk_id, struct timespec *spec) {
//     return -1; // Defaults to 1000000
// }
// struct timespec {
//     WASMRawU64 tv_sec;
//     WASMRawU64 tv_nsec;
// };
// static inline int clock_gettime(int clk_id, struct timespec *spec) {
//     struct timeval tp;
//     struct timezone tzp;
//     gettimeofday(&tp, &tzp);
//     spec->tv_sec = tp.tv_sec;
//     spec->tv_nsec = tp.tv_sec * 1000;
//     return 0;
// }
// static inline clockid_t convert_clockid(nr_wasi_clockid_t in) {
//     return 0;
// }
// #elif defined(_WIN32)

// #if !defined(__MINGW32__)

// #define _AMD64_

// #include "sysinfoapi.h"

// static inline int clock_gettime(int clk_id, struct timespec *spec) {
//     __int64 wintime;
//     GetSystemTimeAsFileTime((FILETIME *)&wintime);
//     wintime -= 116444736000000000i64;            // 1jan1601 to 1jan1970
//     spec->tv_sec = wintime / 10000000i64;        // seconds
//     spec->tv_nsec = wintime % 10000000i64 * 100; // nano-seconds
//     return 0;
// }

// static inline int clock_getres(int clk_id, struct timespec *spec) {
//     return -1; // Defaults to 1000000
// }

// #endif

// static inline clockid_t convert_clockid(nr_wasi_clockid_t in) {
//     return 0;
// }

// #else // _WIN32

// #include <time.h>

// static inline clockid_t convert_clockid(nr_wasi_clockid_t in) {
//     switch (in) {
//     case __WASI_CLOCKID_MONOTONIC:
//         return CLOCK_MONOTONIC;
//     case __WASI_CLOCKID_PROCESS_CPUTIME_ID:
//         return CLOCK_PROCESS_CPUTIME_ID;
//     case __WASI_CLOCKID_REALTIME:
//         return CLOCK_REALTIME;
//     case __WASI_CLOCKID_THREAD_CPUTIME_ID:
//         return CLOCK_THREAD_CPUTIME_ID;
//     default:
//         return (clockid_t)-1;
//     }
// }

// #endif // _WIN32

namespace webrogue {
namespace core {
WASIObject::WASIObject(ModsRuntime *pRuntime, ResourceStorage *resourceStorage,
                       Config *config)
    : runtime(pRuntime), vfs(resourceStorage, config) {
    uvwasi_options_t initOptions;
    uvwasi_errno_t err;

    uvwasi = new uvwasi_s;

    /* Setup the initialization options. */
    initOptions.in = 0;
    initOptions.out = 1;
    initOptions.err = 2;
    initOptions.fd_table_size = 3;
    initOptions.argc = 0;
    initOptions.argv = nullptr;
    static const char *envp[] = {"ENV1=test_env", nullptr};
    initOptions.envp = envp;
    initOptions.preopenc = 0;
    initOptions.preopens = nullptr;
    initOptions.allocator = nullptr;
    initOptions.preopen_socketc = 0;
    initOptions.preopen_sockets = nullptr;

    err = uvwasi_init(uvwasi, &initOptions);

    assert(err == UVWASI_ESUCCESS);
}

WASIObject::~WASIObject() {
    uvwasi_destroy(uvwasi);
    delete uvwasi;
}

WASI_FUNCTION_IMPL(WASMRawI32, environ_get,
                   (WASMRawU32 ptrs, WASMRawU32 buff)) {
    uvwasi_size_t environCount;
    uvwasi_size_t environBufSize;
    uvwasi_errno_t ret =
        uvwasi_environ_sizes_get(uvwasi, &environCount, &environBufSize);
    if (ret != UVWASI_ESUCCESS)
        return WASMRawI32::make(ret);
    std::vector<char *> environment;
    environment.resize(environCount);

    std::vector<char> environBuf;
    environBuf.resize(environBufSize);

    ret = uvwasi_environ_get(uvwasi, environment.data(), environBuf.data());
    if (ret != UVWASI_ESUCCESS)
        return WASMRawI32::make(ret);

    std::vector<WASMRawU32> wasmEnvironment;
    wasmEnvironment.reserve(environCount);
    for (int i = 0; i < environCount; i++) {
        wasmEnvironment.push_back(WASMRawU32::make(
            buff.get() + (environment[i] - environBuf.data())));
    }

    WASI_CHECK(runtime->setVMData(wasmEnvironment.data(), ptrs.get(),
                                  sizeof(WASMI32) * wasmEnvironment.size()));
    WASI_CHECK(
        runtime->setVMData(environBuf.data(), buff.get(), environBuf.size()));

    return WASMRawI32::make(ret);
}

WASI_FUNCTION_IMPL(WASMRawI32, environ_sizes_get,
                   (WASMRawI32 count_offset, WASMRawI32 buffsize_offset)) {
    uvwasi_size_t environCount;
    uvwasi_size_t environBufSize;
    uvwasi_errno_t ret =
        uvwasi_environ_sizes_get(uvwasi, &environCount, &environBufSize);
    WASMU32 data = WASMU32::make(environCount);
    WASI_CHECK(runtime->setVMData(&data, count_offset.get(), sizeof(WASMI32)));
    data = WASMU32::make(environBufSize);
    WASI_CHECK(
        runtime->setVMData(&data, buffsize_offset.get(), sizeof(WASMI32)));
    return WASMRawI32::make(ret);
}

WASI_FUNCTION_IMPL(WASMRawI32, random_get, (WASMRawU32 a, WASMRawU32 b)) {
    return WASMRawI32::make(0);
}

WASI_FUNCTION_IMPL(WASMRawU32, clock_time_get,
                   (WASMRawU32 clk_id, WASMRawU64 precision,
                    WASMRawU32 out_time_offset)) {

    // // m3ApiGetArg(__wasi_clockid_t, wasi_clk_id)
    // //     m3ApiGetArg(__wasi_timestamp_t, precision)
    // //         m3ApiGetArgMem(__wasi_timestamp_t *, time)

    // //             m3ApiCheckMem(time, sizeof(__wasi_timestamp_t));

    // clockid_t clk = convert_clockid(clk_id);
    // if (clk < 0)
    //     return 28;

    // struct timespec tp;
    // if (clock_gettime(clk, &tp) != 0) {
    //     return 8;
    // }

    // WASI_CHECK(
    //     runtime->setVMData(&tp, out_time_offset,
    //     sizeof(nr_wasi_timestamp_t)));
    // // m3ApiWriteMem64(time, convert_timespec(&tp));
    return WASMRawU32::make(0);
}

WASI_FUNCTION_IMPL(WASMRawI32, fd_sync, (WASMRawU32 a)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, fd_allocate,
                   (WASMRawU32 a, WASMRawU64 b, WASMRawU64 c)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, fd_datasync, (WASMRawU32 a)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, fd_tell, (WASMRawU32 a, WASMRawU32 b)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, fd_pwrite,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU64 d,
                    WASMRawU32 e)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, clock_res_get, (WASMRawU32 a, WASMRawU32 b)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, fd_renumber, (WASMRawU32 a, WASMRawU32 b)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, fd_pread,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU64 d,
                    WASMRawU32 e)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawU32, path_create_directory,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, sched_yield, ()) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, fd_filestat_set_times,
                   (WASMRawU32 a, WASMRawU64 b, WASMRawU64 c, WASMRawU32 d)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, args_sizes_get, (WASMRawU32 a, WASMRawU32 b)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, fd_filestat_set_size,
                   (WASMRawU32 a, WASMRawU64 b)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, sock_recv,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU32 d,
                    WASMRawU32 e, WASMRawU32 f)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, fd_filestat_get, (WASMRawU32 a, WASMRawU32 b)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, sock_shutdown, (WASMRawU32 a, WASMRawU32 b)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, poll_oneoff,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU32 d)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, path_link,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU32 d,
                    WASMRawU32 e, WASMRawU32 f, WASMRawU32 g)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, sock_accept,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, path_readlink,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU32 d,
                    WASMRawU32 e, WASMRawU32 f)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, fd_advise,
                   (WASMRawU32 a, WASMRawU64 b, WASMRawU64 c, WASMRawU32 d)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, path_filestat_set_times,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU32 d,
                    WASMRawU64 e, WASMRawU64 f, WASMRawU32 g)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, args_get, (WASMRawU32 a, WASMRawU32 b)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, path_symlink,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU32 d,
                    WASMRawU32 e)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, sock_send,
                   (WASMRawU32 a, WASMRawU32 b, WASMRawU32 c, WASMRawU32 d,
                    WASMRawU32 e)) {
    abort();
}
WASI_FUNCTION_IMPL(WASMRawI32, fd_fdstat_set_rights,
                   (WASMRawU32 a, WASMRawU64 b, WASMRawU64 c)) {
    abort();
}

} // namespace core
} // namespace webrogue
