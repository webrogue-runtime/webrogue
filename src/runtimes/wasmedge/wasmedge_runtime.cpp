#include "wasmedge_runtime.hpp"
#include "../../core/CompactLinking.hpp"

#include "spdlog/common.h"
#include "spdlog/spdlog.h"
#include "wasmedge/wasmedge.h"

#include "../../common/stringize.hpp"
#include "../../core/ApiObject.hpp"
#include "../../core/ConsoleStream.hpp"
#include "wasmedge_templates.hpp"
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iterator>
#include <vector>

namespace webrogue {
namespace runtimes {
namespace wasmedge {

static inline WasmEdge_Result
procExitFunc(void *data, const WasmEdge_CallingFrameContext *callFrameCxt,
             const WasmEdge_Value *in, WasmEdge_Value *out) {
    auto *runtime = reinterpret_cast<core::ModsRuntime *>(data);
    runtime->procExit = true;
    runtime->procExitCode = WasmEdge_ValueGetI32(in[0]);
    return WasmEdge_Result_Terminate;
}

bool readRealFile(std::vector<uint8_t> &out, std::string path) {
    std::ifstream file(path, std::ios::in | std::ios::binary);
    if (!file.is_open())
        return false;
    file.unsetf(std::ios::skipws);
    file.seekg(0, std::ios_base::end);
    size_t const fileSize = file.tellg();
    file.seekg(0, std::ios_base::beg);
    out.resize(0);
    out.reserve(fileSize);
    out.insert(out.begin(), std::istream_iterator<uint8_t>(file),
               std::istream_iterator<uint8_t>());
    return true;
}

WasmEdgeModsRuntime::WasmEdgeModsRuntime(
    webrogue::core::ResourceStorage *resourceStorage,
    webrogue::core::Config const *config)
    : ModsRuntime(resourceStorage, config) {
}

void WasmEdgeModsRuntime::initMods() {
    spdlog::set_level(spdlog::level::err);
    auto linkResult = core::getCompactlyLinkedBinaries(this, resourceStorage,
                                                       config, [this]() {
                                                           interrupt();
                                                       });
    WasmEdge_Result res;
    confCxt = WasmEdge_ConfigureCreate();
    std::string const aotCachePath =
        config->getDataPath() + "/wasmedge_aot_cache";
    if (true) {
        // *wrout << "precompiling...\n";
        WasmEdge_CompilerContext *compCtx = WasmEdge_CompilerCreate(confCxt);
        res = WasmEdge_CompilerCompileFromBuffer(compCtx, linkResult->data(),
                                                 linkResult->size(),
                                                 aotCachePath.c_str());
        if (!WasmEdge_ResultOK(res)) {
            assert(false);
            return;
        }
        WasmEdge_CompilerDelete(compCtx);
    }
    vmCxt = WasmEdge_VMCreate(confCxt, NULL);
    std::vector<uint8_t> const wasm;

    res = WasmEdge_VMLoadWasmFromFile(vmCxt, aotCachePath.c_str());
    if (!WasmEdge_ResultOK(res)) {
        assert(false);
        return;
    }

    res = WasmEdge_VMValidate(vmCxt);
    if (!WasmEdge_ResultOK(res)) {
        assert(false);
        return;
    }
    { // wr_api
        WasmEdge_String const hostName =
            WasmEdge_StringCreateByCString("webrogue");
        WasmEdge_ModuleInstanceContext *HostMod =
            WasmEdge_ModuleInstanceCreate(hostName);
        WasmEdge_StringDelete(hostName);

#define WR_API_FUNCTION(RET_TYPE, NAME, ARGS)                                  \
    FuncLinker<decltype(&core::ApiObject::NAME),                               \
               &core::ApiObject::NAME>::link("webrogue", stringize(NAME),      \
                                             HostMod, this);
#include "../../../mods/core/include/common/wr_api_functions.def"
#undef WR_API_FUNCTION

        WasmEdge_StoreContext *storeCxt = WasmEdge_StoreCreate();
        WasmEdge_ExecutorContext *execCxt = WasmEdge_ExecutorCreate(NULL, NULL);
        res = WasmEdge_ExecutorRegisterImport(execCxt, storeCxt, HostMod);
        if (!WasmEdge_ResultOK(res)) {
            assert(false);
            return;
        }

        res = WasmEdge_VMRegisterModuleFromImport(vmCxt, HostMod);
        if (!WasmEdge_ResultOK(res)) {
            assert(false);
            return;
        }

        WasmEdge_StoreDelete(storeCxt);   // ?
        WasmEdge_ExecutorDelete(execCxt); // ?
        // WasmEdge_ModuleInstanceDelete(HostMod);
    }
    { // wasi_api
        WasmEdge_String const hostName =
            WasmEdge_StringCreateByCString("wasi_snapshot_preview1");
        WasmEdge_ModuleInstanceContext *hostMod =
            WasmEdge_ModuleInstanceCreate(hostName);
        WasmEdge_StringDelete(hostName);

#define WASI_FUNCTION(RET_TYPE, NAME, ARGS)                                    \
    FuncLinker<decltype(&core::WASIObject::NAME),                              \
               &core::WASIObject::NAME>::link("wasi_snapshot_preview1",        \
                                              stringize(NAME), hostMod, this);
#include "../../core/wasi_functions.def"
#undef WASI_FUNCTION
        {
            WasmEdge_ValType paramTypes[] = {WasmEdge_ValTypeGenI32()};
            WasmEdge_FunctionTypeContext *hostFType =
                WasmEdge_FunctionTypeCreate(paramTypes, 1, nullptr, 0);
            WasmEdge_FunctionInstanceContext *hostFunc =
                WasmEdge_FunctionInstanceCreate(hostFType, procExitFunc, this,
                                                0);
            WasmEdge_FunctionTypeDelete(hostFType);

            WasmEdge_String const hostName =
                WasmEdge_StringCreateByCString("proc_exit");
            WasmEdge_ModuleInstanceAddFunction(hostMod, hostName, hostFunc);
            WasmEdge_StringDelete(hostName);
        }

        WasmEdge_StoreContext *storeCxt = WasmEdge_StoreCreate();
        WasmEdge_ExecutorContext *execCxt = WasmEdge_ExecutorCreate(NULL, NULL);
        res = WasmEdge_ExecutorRegisterImport(execCxt, storeCxt, hostMod);
        if (!WasmEdge_ResultOK(res)) {
            assert(false);
            return;
        }

        res = WasmEdge_VMRegisterModuleFromImport(vmCxt, hostMod);
        if (!WasmEdge_ResultOK(res)) {
            assert(false);
            return;
        }

        WasmEdge_StoreDelete(storeCxt);   // ?
        WasmEdge_ExecutorDelete(execCxt); // ?
        // WasmEdge_ModuleInstanceDelete(HostMod);
    }

    res = WasmEdge_VMInstantiate(vmCxt);
    if (!WasmEdge_ResultOK(res)) {
        assert(false);
        return;
    }

    // *wrout << "initializing mods...\n";
    WasmEdge_String funcName =
        WasmEdge_StringCreateByCString("__wasm_call_ctors");
    res = WasmEdge_VMExecute(vmCxt, funcName, nullptr, 0, nullptr, 0);
    WasmEdge_StringDelete(funcName);
    if (!WasmEdge_ResultOK(res)) {
        assert(false);
        return;
    }

    for (std::string const modName : resourceStorage->modNames) {
        std::string const stdFuncName = "init_mod_" + modName;
        funcName = WasmEdge_StringCreateByCString(stdFuncName.c_str());

        res = WasmEdge_VMExecute(vmCxt, funcName, nullptr, 0, nullptr, 0);
        WasmEdge_StringDelete(funcName);
        if (!WasmEdge_ResultOK(res)) {
            assert(false);
            return;
        }
    }
    // *wrout << "all mods initialized\n";

    isInitialized = true;
}
void WasmEdgeModsRuntime::start() {
    WasmEdge_String const funcName = WasmEdge_StringCreateByCString("wr_start");

    WasmEdge_Result const res =
        WasmEdge_VMExecute(vmCxt, funcName, nullptr, 0, nullptr, 0);
    WasmEdge_StringDelete(funcName);
    if (!WasmEdge_ResultOK(res)) {
        assert(false);
        return;
    }
}

bool WasmEdgeModsRuntime::getVMData(void *outPtr, uint64_t offset,
                                    int32_t size) {
    const WasmEdge_CallingFrameContext *callFrameCxt =
        (const WasmEdge_CallingFrameContext *)vmContext;
    WasmEdge_MemoryInstanceContext *memCxt =
        WasmEdge_CallingFrameGetMemoryInstance(callFrameCxt, 0);
    WasmEdge_Result const res = WasmEdge_MemoryInstanceGetData(
        memCxt, (uint8_t *)(outPtr), offset, size);
    return WasmEdge_ResultOK(res);
}
bool WasmEdgeModsRuntime::setVMData(const void *inPtr, uint64_t offset,
                                    int32_t size) {
    const WasmEdge_CallingFrameContext *callFrameCxt =
        (const WasmEdge_CallingFrameContext *)vmContext;
    WasmEdge_MemoryInstanceContext *memCxt =
        WasmEdge_CallingFrameGetMemoryInstance(callFrameCxt, 0);
    WasmEdge_Result const res = WasmEdge_MemoryInstanceSetData(
        memCxt, (uint8_t *)(inPtr), offset, size);
    return WasmEdge_ResultOK(res);
}
size_t WasmEdgeModsRuntime::vmSize() {
    const WasmEdge_CallingFrameContext *callFrameCxt =
        (const WasmEdge_CallingFrameContext *)vmContext;
    WasmEdge_MemoryInstanceContext *memCxt =
        WasmEdge_CallingFrameGetMemoryInstance(callFrameCxt, 0);
    return WasmEdge_MemoryInstanceGetPageSize(memCxt);
}
size_t WasmEdgeModsRuntime::voidptrSize() {
    return 4;
};

WasmEdgeModsRuntime::~WasmEdgeModsRuntime() {
    WasmEdge_VMDelete(vmCxt);
    WasmEdge_ConfigureDelete(confCxt);
}
} // namespace wasmedge
} // namespace runtimes
} // namespace webrogue
