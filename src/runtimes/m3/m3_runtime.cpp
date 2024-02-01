#include "m3_runtime.hpp"

#include "../../core/ApiObject.hpp"
#include "../../core/CompactLinking.hpp"
#include "m3_templates.hpp"
#include <cstdint>
#include <cstring>
#include <memory>
#include <string>

using namespace webrogue::core;

#define WR_API_FUNCTION(RET_TYPE, NAME, ARGS) extern "C" RET_TYPE NAME ARGS;
#include "../../../mods/core/include/common/wr_api_functions.def"
#undef WR_API_FUNCTION
#include "../../common/stringize.hpp"

namespace webrogue {
namespace runtimes {
namespace m3 {

m3ApiRawFunction(proc_exit) {
    m3ApiGetArg(uint32_t, code)

        auto *modsRuntime = reinterpret_cast<M3ModsRuntime *>(_ctx->userdata);
    modsRuntime->procExit = true;
    modsRuntime->procExitCode = code;

    m3ApiTrap(m3Err_trapExit);
}

M3ModsRuntime::M3ModsRuntime(webrogue::core::ConsoleStream *wrout,
                             webrogue::core::ConsoleStream *wrerr,
                             webrogue::core::ResourceStorage *resourceStorage,
                             webrogue::core::Config *config)
    : ModsRuntime(wrout, wrerr, resourceStorage, config){};

void M3ModsRuntime::initMods() {

    linkedWasm = core::getCompactlyLinkedBinaries(
        this, resourceStorage, config,
        [this]() {
            interrupt();
        },
        wrout, wrerr);
    if (!linkedWasm) {
        *wrerr << "linking failed\n";
        return;
    }

    modsEnvironment = m3_NewEnvironment();
    if (!modsEnvironment) {
        *wrerr << "initializing m3 environment failed\n";
        return;
    }
    modsRuntime = m3_NewRuntime(modsEnvironment, 64 * 1024 * 1024, nullptr);
    if (!modsRuntime) {
        *wrerr << "initializing m3 runtime failed\n";
        return;
    }
    M3Result err = m3_ParseModule(modsEnvironment, &modsModule,
                                  linkedWasm->data(), linkedWasm->size());
    if (err) {
        *wrerr << "parsing wasm binnary failed: " << err << "\n";
        return;
    }

    m3_SetModuleName(modsModule, "mods");
    err = m3_LoadModule(modsRuntime, modsModule);
    if (err) {
        *wrerr << "loading wasm module failed: " << err << "\n";
        return;
    }

#define WR_API_FUNCTION(RET_TYPE, NAME, ARGS)                                  \
    FuncLinker<decltype(&core::ApiObject::NAME),                               \
               &core::ApiObject::NAME>::link("webrogue", stringize(NAME),      \
                                             modsModule, this);
#include "../../../mods/core/include/common/wr_api_functions.def"
#undef WR_API_FUNCTION

#define WASI_FUNCTION(RET_TYPE, NAME, ARGS)                                    \
    FuncLinker<decltype(&core::WASIObject::NAME),                              \
               &core::WASIObject::NAME>::link("wasi_snapshot_preview1",        \
                                              stringize(NAME), modsModule,     \
                                              this);
#include "../../core/wasi_functions.def"
#undef WASI_FUNCTION

    m3_LinkRawFunctionEx(modsModule, "wasi_snapshot_preview1", "proc_exit",
                         "v(i)", proc_exit, (void *)(this));
    *wrout << "initializing mods...\n";

    IM3Function func;

    err = m3_FindFunction(&func, modsRuntime, "__wasm_call_ctors");
    if (err) {
        *wrerr << err << "\n";
        return;
    }
    err = m3_Call(func, 0, nullptr);
    if (err) {
        *wrerr << err << "\n";
        return;
    }
    for (std::string modName : resourceStorage->modNames) {
        std::string funcName = "init_mod_" + modName;

        err = m3_FindFunction(&func, modsRuntime, funcName.c_str());
        if (err) {
            *wrerr << err << "\n";
            return;
        }
        err = m3_Call(func, 0, nullptr);
        if (err) {
            *wrerr << err << "\n";
            return;
        }
    }
    *wrout << "all mods initialized\n";

    err = m3_FindFunction(&startFunction, modsRuntime, "wr_start");
    if (err) {
        *wrerr << err << "\n";
        return;
    }

    isInitialized = true;
}
void M3ModsRuntime::start() {
    M3Result err = m3_Call(startFunction, 0, nullptr);
    if (err && !procExit) {
        *wrerr << err << "\n";
        return;
    }
}
bool M3ModsRuntime::getVMData(void *outPtr, uint64_t offset, int32_t size) {
    uint32_t currentMemSize;
    uint8_t *currentMem = m3_GetMemory(modsRuntime, &currentMemSize, 0);
    bool const memOk = offset >= 0 && (offset + size) <= currentMemSize;
    if (!memOk)
        return false;
    memcpy(outPtr, currentMem + offset, size);
    return true;
}
bool M3ModsRuntime::setVMData(const void *inPtr, uint64_t offset,
                              int32_t size) {
    uint32_t currentMemSize;
    uint8_t *currentMem = m3_GetMemory(modsRuntime, &currentMemSize, 0);
    bool const memOk = offset >= 0 && (offset + size) <= currentMemSize;
    if (!memOk)
        return false;
    memcpy(currentMem + offset, inPtr, size);
    return true;
}
size_t M3ModsRuntime::vmSize() {
    return m3_GetMemorySize(modsRuntime);
}
size_t M3ModsRuntime::voidptrSize() {
    return 4;
};
M3ModsRuntime::~M3ModsRuntime() {
    if (modsRuntime)
        m3_FreeRuntime(modsRuntime);
    if (modsEnvironment)
        m3_FreeEnvironment(modsEnvironment);
}
} // namespace m3
} // namespace runtimes
} // namespace webrogue
