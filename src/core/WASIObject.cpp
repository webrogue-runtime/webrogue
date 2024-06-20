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

#if defined(_WIN32)
#include <io.h>
#endif

namespace webrogue {
namespace core {

#if defined(_WIN32)
int win_stdio_fd(DWORD stdio_type, bool rdonly) {
    HANDLE handle = GetStdHandle(stdio_type);
    if (handle == INVALID_HANDLE_VALUE || !handle) {
        handle =
            CreateFile("nul", rdonly ? GENERIC_READ : GENERIC_WRITE,
                       rdonly ? FILE_SHARE_READ : 0, NULL, OPEN_EXISTING,
                       rdonly ? FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED
                              : FILE_ATTRIBUTE_NORMAL,
                       NULL);
    }
    return _open_osfhandle((intptr_t)handle, rdonly ? _O_RDONLY : _O_WRONLY);
}
#endif

WASIObject::WASIObject(ModsRuntime *pRuntime, ResourceStorage *resourceStorage,
                       Config *config)
    : runtime(pRuntime) {
    uvwasi_options_t initOptions;
    uvwasi_options_init(&initOptions);
    uvwasi_errno_t err;

    uvwasi = new uvwasi_s;

#if defined(_WIN32)
    initOptions.in = win_stdio_fd(STD_INPUT_HANDLE, true);
    initOptions.out = win_stdio_fd(STD_OUTPUT_HANDLE, false);
    initOptions.err = win_stdio_fd(STD_ERROR_HANDLE, false);
#else
    initOptions.in = 0;
    initOptions.out = 1;
    initOptions.err = 2;
#endif
    initOptions.fd_table_size = 3;
    initOptions.argc = 0;
    initOptions.argv = nullptr;
    static const char *envp[] = {"ENV1=test_env", nullptr};
    initOptions.envp = envp;
    std::vector<uvwasi_preopen_t> preopens;
    preopens.push_back({
        "/",                     // mapped_path
        config->dataPath.c_str() // real_path
    });
    preopens.push_back({
        "./",                    // mapped_path
        config->dataPath.c_str() // real_path
    });
    initOptions.preopenc = preopens.size();
    initOptions.preopens = preopens.data();
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
                   (WASMRawU32 wasi_clk_id, WASMRawU64 precision,
                    WASMRawU32 time)) {
    if (!runtime->isVMRangeValid(time.get(), sizeof(uvwasi_timestamp_t)))
        return E_BAD_ADDR;

    uvwasi_timestamp_t t;
    uvwasi_errno_t const ret =
        uvwasi_clock_time_get(uvwasi, wasi_clk_id.get(), precision.get(), &t);
    runtime->setVMInt<uint64_t>(time.get(), t);
    return WASMRawU32::make(ret);
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
WASI_FUNCTION_IMPL(WASMRawU32, proc_raise, (WASMRawU32 a)) {
    abort();
}

} // namespace core
} // namespace webrogue
