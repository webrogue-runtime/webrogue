#include "wasm2c_runtime.hpp"
#include "../../core/ModsRuntime.hpp"
#include <cstdint>
#include <cstdlib>
#if __cpp_exceptions
#include <exception>
#endif
#include <memory>
#include <stdio.h>
#include <stdlib.h>

namespace webrogue {
namespace runtimes {
namespace wasm2c {
#if __cpp_exceptions
class ExitException : public std::exception {};
#endif

extern "C" void
w2c_wasi__snapshot__preview1_proc_exit(struct w2c_wasi__snapshot__preview1 *env,
                                       u32 exitCode) {
    auto *runtime =
        reinterpret_cast<webrogue::runtimes::wasm2c::Wasm2cModsRuntime *>(env);
    runtime->procExit = true;
    runtime->procExitCode = exitCode;
#if __cpp_exceptions
    throw ExitException();
#else
    abort();
#endif
}
Wasm2cModsRuntime::Wasm2cModsRuntime(
    webrogue::core::ConsoleStream *wrout, webrogue::core::ConsoleStream *wrerr,
    webrogue::core::ResourceStorage *resourceStorage,
    webrogue::core::Config *config)
    : ModsRuntime(wrout, wrerr, resourceStorage, config) {
}
void Wasm2cModsRuntime::initMods() {
    wasm_rt_init();
    wasm2c_linked_instantiate(&linked, (w2c_wasi__snapshot__preview1 *)this,
                              (w2c_webrogue *)this);
    w2c_linked_0x5F_wasm_call_ctors(&linked);
#define mod_to_embed(name) w2c_linked_init_mod_##name(&linked);
#include stringize(WASM2C_WEBROGUE_MOD_LIST_HEADER)
#undef mod_to_embed
    isInitialized = true;
};
void Wasm2cModsRuntime::start() {
#if __cpp_exceptions
    try {
#endif
        w2c_linked_wr_start(&linked);
#if __cpp_exceptions
    } catch (ExitException &) {
    }
#endif
    wasm2c_linked_free(&linked);
};
bool Wasm2cModsRuntime::getVMData(void *outPtr, uint64_t offset, int32_t size) {
    if (offset < 0 || offset + size > linked.w2c_memory.size)
        return false;
    memcpy(outPtr, linked.w2c_memory.data + offset, size);
    return true;
}
bool Wasm2cModsRuntime::setVMData(const void *inPtr, uint64_t offset,
                                  int32_t size) {
    if (offset < 0 || offset + size > linked.w2c_memory.size)
        return false;
    memcpy(linked.w2c_memory.data + offset, inPtr, size);
    return true;
}
size_t Wasm2cModsRuntime::voidptrSize() {
    return 4;
};
} // namespace wasm2c
} // namespace runtimes
} // namespace webrogue
