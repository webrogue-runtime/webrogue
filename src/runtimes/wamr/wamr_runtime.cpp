#include "wamr_runtime.hpp"
#include "../../../external/wamr/core/iwasm/include/wasm_export.h"
#include "../../common/stringize.hpp"
#include "../../core/CompactLinking.hpp"
#include "../../core/ModsRuntime.hpp"
#include "wamr_templates.hpp"
#include <cstddef>
#include <cstdint>
#include <cstring>

namespace webrogue {
namespace runtimes {
namespace wamr {

WamrModsRuntime::WamrModsRuntime(
    webrogue::core::ConsoleStream *wrout, webrogue::core::ConsoleStream *wrerr,
    webrogue::core::ResourceStorage *resourceStorage,
    webrogue::core::Config *config)
    : ModsRuntime(wrout, wrerr, resourceStorage, config) {
}
static void procExitFunc(wasm_exec_env_t execEnv, uint32_t exitCode) {
    wasm_module_inst_t moduleInst = get_module_inst(execEnv);
    auto *runtime = reinterpret_cast<webrogue::core::ModsRuntime *>(
        wasm_runtime_get_user_data(execEnv));
    runtime->procExit = true;
    runtime->procExitCode = exitCode;
    wasm_runtime_set_exception(moduleInst, "wasi proc exit");
}

NativeSymbol nativeSymbols[] = {
#define WR_API_FUNCTION(RET_TYPE, NAME, ARGS)                                  \
                                                                               \
    FuncLinker<decltype(&core::ApiObject::NAME),                               \
               &core::ApiObject::NAME>::nativeSymbol(stringize(NAME)),
#include "../../../mods/core/include/common/wr_api_functions.def"
#undef WR_API_FUNCTION
};

NativeSymbol wasiSymbols[] = {
#define WASI_FUNCTION(RET_TYPE, NAME, ARGS)                                    \
                                                                               \
    FuncLinker<decltype(&core::WASIObject::NAME),                              \
               &core::WASIObject::NAME>::nativeSymbol(stringize(NAME)),
#include "../../core/wasi_functions.def"
#undef WASI_FUNCTION
    {"proc_exit", (void *)procExitFunc, "(i)"},
};

void WamrModsRuntime::initMods() {
    auto linked = core::getCompactlyLinkedBinaries(
        this, resourceStorage, config,
        [this]() {
            interrupt();
        },
        wrout, wrerr);
    if (!linked || linked->size() == 0) {
        *wrerr << "linking failed\n";
        return;
    }
    *wrout << "initializing runtime for mods...\n";
    RuntimeInitArgs initArgs;
    memset(&initArgs, 0, sizeof(RuntimeInitArgs));
    initArgs.fast_jit_code_cache_size = linked->size() * 8;
    if (wasm_runtime_is_running_mode_supported(Mode_Fast_JIT))
        initArgs.running_mode = Mode_Fast_JIT;
    else
        initArgs.running_mode = Mode_Interp;
    initArgs.mem_alloc_type = Alloc_With_System_Allocator;
    if (!wasm_runtime_full_init(&initArgs))
        return;
    initializedWasmRuntime = true;
    char errorBuf[128];
    wasm_function_inst_t func;
    *wrout << "loading linked mods...\n";

    if (!wasm_runtime_register_natives("wasi_snapshot_preview1", wasiSymbols,
                                       sizeof(wasiSymbols) /
                                           sizeof(NativeSymbol))) {
        return;
    }
    if (!wasm_runtime_register_natives("webrogue", nativeSymbols,
                                       sizeof(nativeSymbols) /
                                           sizeof(NativeSymbol))) {
        return;
    }
    module = wasm_runtime_load(linked->data(), linked->size(), errorBuf,
                               sizeof(errorBuf));
    if (!module) {
        *wrerr << "Error while loading linked module: " << errorBuf << "\n";
        return;
    }
    moduleInst = wasm_runtime_instantiate(module, stackSize, heapSize, errorBuf,
                                          sizeof(errorBuf));
    if (!moduleInst) {
        *wrerr << errorBuf;
        return;
    }
    execEnv = wasm_runtime_create_exec_env(moduleInst, stackSize);
    wasm_runtime_set_user_data(execEnv, this);
    *wrout << "initializing mods...\n";
    func = wasm_runtime_lookup_function(moduleInst, "__wasm_call_ctors");
    if (!wasm_runtime_call_wasm_v(execEnv, func, 0, nullptr, 0)) {
        return;
    }
    for (std::string const modName : resourceStorage->modNames) {
        std::string const funcName = "init_mod_" + modName;
        func = wasm_runtime_lookup_function(moduleInst, funcName.c_str());
        if (!wasm_runtime_call_wasm_v(execEnv, func, 0, nullptr, 0)) {
            return;
        }
    }
    *wrout << "all mods initialized\n";
    startFunc = wasm_runtime_lookup_function(moduleInst, "wr_start");

    isInitialized = true;
};
void WamrModsRuntime::start() {
    if (!isInitialized)
        return;
    bool const ret =
        wasm_runtime_call_wasm_v(execEnv, startFunc, 0, nullptr, 0);
    if (procExit) {
        wasm_runtime_clear_exception(moduleInst);
    } else if (!ret) {
        *wrerr << "Error while executing wr_start: "
               << wasm_runtime_get_exception(moduleInst) << "\n";
        isInitialized = false;
    }
};

bool WamrModsRuntime::getVMData(void *outPtr, uint64_t offset, int32_t size) {
    uint64_t currentMemSize;
    uint8_t *currentMem =
        (uint8_t *)wasm_runtime_addr_app_to_native(moduleInst, 0);

    wasm_runtime_get_app_addr_range(moduleInst, 0, nullptr, &currentMemSize);
    bool const memOk = offset >= 0 && (offset + size) <= currentMemSize;
    if (!memOk)
        return false;
    memcpy(outPtr, currentMem + offset, size);
    return true;
}
bool WamrModsRuntime::setVMData(const void *inPtr, uint64_t offset,
                                int32_t size) {
    uint64_t currentMemSize;
    uint8_t *currentMem =
        (uint8_t *)wasm_runtime_addr_app_to_native(moduleInst, 0);

    wasm_runtime_get_app_addr_range(moduleInst, 0, nullptr, &currentMemSize);
    bool const memOk = offset >= 0 && (offset + size) <= currentMemSize;
    if (!memOk)
        return false;
    memcpy(currentMem + offset, inPtr, size);
    return true;
}
size_t WamrModsRuntime::vmSize() {
    uint64_t currentMemSize;
    uint8_t *currentMem =
        (uint8_t *)wasm_runtime_addr_app_to_native(moduleInst, 0);

    wasm_runtime_get_app_addr_range(moduleInst, 0, nullptr, &currentMemSize);
    return currentMemSize;
}
size_t WamrModsRuntime::voidptrSize() {
    return 4;
};

WamrModsRuntime::~WamrModsRuntime() {
    if (execEnv)
        wasm_runtime_destroy_exec_env(execEnv);
    if (moduleInst)
        wasm_runtime_deinstantiate(moduleInst);
    if (module)
        wasm_runtime_unload(module);
    if (initializedWasmRuntime)
        wasm_runtime_destroy();
}
} // namespace wamr
} // namespace runtimes
} // namespace webrogue
