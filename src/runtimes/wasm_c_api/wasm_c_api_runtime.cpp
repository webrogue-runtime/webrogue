#include "wasm_c_api_runtime.hpp"
#include "../../common/stringize.hpp"
#include "../../core/ApiObject.hpp"
#include "../../core/CompactLinking.hpp"
#include "../../core/ConsoleStream.hpp"
#include "wasm.h"
#include "wasm_c_api_templates.hpp"
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iterator>
#include <map>
#include <string>
#include <vector>

namespace webrogue {
namespace runtimes {
namespace wasm_c_api {

static const std::string procExitMessage = "proc_exit";

static wasm_trap_t *procExitFunc(void *env, const wasm_val_vec_t *args,
                                 wasm_val_vec_t *results) {
    auto *runtime =
        reinterpret_cast<webrogue::runtimes::wasm_c_api::WasmCApiModsRuntime *>(
            env);
    runtime->procExit = true;
    runtime->procExitCode = args->data[0].of.f32;
    auto *trapMessage = new wasm_name_t();
    auto *copiedMessage = new std::string(procExitMessage);
    trapMessage->data = const_cast<char *>(
        reinterpret_cast<const char *>(copiedMessage->data()));
    trapMessage->size = copiedMessage->size();
    return wasm_trap_new(runtime->store, trapMessage);
}

WasmCApiModsRuntime::WasmCApiModsRuntime(
    webrogue::core::ConsoleStream *wrout, webrogue::core::ConsoleStream *wrerr,
    webrogue::core::ResourceStorage *resourceStorage,
    webrogue::core::Config *config)
    : ModsRuntime(wrout, wrerr, resourceStorage, config) {
}

void WasmCApiModsRuntime::initMods() {
    auto linkResult = core::getCompactlyLinkedBinaries(
        this, resourceStorage, config,
        [this]() {
            interrupt();
        },
        wrout, wrerr);

    *wrout << "creating engine...\n";
    engine = wasm_engine_new();
    *wrout << "creating store...\n";
    store = wasm_store_new(engine);
    wasm_byte_vec_t binary;
    wasm_byte_vec_new_uninitialized(&binary, linkResult->size());
    memcpy(binary.data, linkResult->data(), linkResult->size());
    *wrout << "creating module...\n";
    module = wasm_module_new(store, &binary);

    if (!module) {
        *wrerr << "module == nullptr\n";
        return;
    }
    std::map<std::string, wasm_extern_t *> importMap;

#define WR_API_FUNCTION(RET_TYPE, NAME, ARGS)                                  \
    FuncLinker<decltype(&core::ApiObject::NAME),                               \
               &core::ApiObject::NAME>::link("webrogue", stringize(NAME),      \
                                             store, this, importMap);
#include "../../../mods/core/include/common/wr_api_functions.def"
#undef WR_API_FUNCTION

#define WASI_FUNCTION(RET_TYPE, NAME, ARGS)                                    \
    FuncLinker<decltype(&core::WASIObject::NAME),                              \
               &core::WASIObject::NAME>::link("wasi_snapshot_preview1",        \
                                              stringize(NAME), store, this,    \
                                              importMap);
#include "../../core/wasi_functions.def"
#undef WASI_FUNCTION
    {
        wasm_valtype_vec_t argumentTypes;
        wasm_valtype_vec_new_uninitialized(&argumentTypes, 1);
        argumentTypes.data[0] = wasm_valtype_new_i32();
        wasm_valtype_vec_t resultTypes;
        wasm_valtype_vec_new_uninitialized(&resultTypes, 0);
        wasm_functype_t *wrappedFuncType =
            wasm_functype_new(&argumentTypes, &resultTypes);
        wasm_func_t *wrappedWasmFuncType = wasm_func_new_with_env(
            store, wrappedFuncType, procExitFunc, this, NULL);
        importMap["wasi_snapshot_preview1.proc_exit"] =
            wasm_func_as_extern(wrappedWasmFuncType);
    }
    wasm_importtype_vec_t expectedImports;

    wasm_module_imports(module, &expectedImports);
    std::vector<wasm_extern_t *> externs;
    externs.reserve(expectedImports.size);
    for (size_t i = 0; i < expectedImports.size; i++) {
        wasm_importtype_t *expectedImport = expectedImports.data[i];
        assert(wasm_externtype_kind(wasm_importtype_type(expectedImport)) ==
               WASM_EXTERN_FUNC);
        const wasm_name_t *wasmFuncName = wasm_importtype_name(expectedImport);
        const wasm_name_t *wasmModuleName =
            wasm_importtype_module(expectedImport);
        std::string const importModuleAndName =
            std::string(wasmModuleName->data, wasmModuleName->size) + "." +
            std::string(wasmFuncName->data, wasmFuncName->size);
        assert(importMap.count(importModuleAndName));
        externs.push_back(importMap[importModuleAndName]);
    }

    wasm_extern_vec_t const imports = {externs.size(), externs.data()};
    *wrout << "creating instance...\n";
    wasm_trap_t *trap;
    instance = wasm_instance_new(store, module, &imports, &trap);
    if (!instance) {
        wasm_message_t errMessage;
        wasm_trap_message(trap, &errMessage);
        *wrerr << "instance == nullptr\n"
               << std::string(errMessage.data, errMessage.size) << "\n";
        return;
    }

    wasm_exporttype_vec_t exportTypes;
    exports = new wasm_extern_vec_t;
    wasm_module_exports(module, &exportTypes);
    wasm_instance_exports(instance, exports);

    std::map<std::string, const wasm_func_t *> exportedFuncMap;

    for (size_t i = 0; i < exports->size; ++i) {
        switch (
            wasm_externtype_kind(wasm_exporttype_type(exportTypes.data[i]))) {
        case WASM_EXTERN_FUNC: {
            const wasm_name_t *wasmFuncName =
                wasm_exporttype_name(exportTypes.data[i]);
            std::string const stdFuncName =
                std::string(wasmFuncName->data, wasmFuncName->size);
            exportedFuncMap[stdFuncName] =
                wasm_extern_as_func(exports->data[i]);
            break;
        };
        case WASM_EXTERN_MEMORY: {
            memory = wasm_extern_as_memory(exports->data[i]);
            break;
        };
        }
    }

    *wrout << "initializing mods...\n";
    wasm_val_vec_t const args = WASM_EMPTY_VEC;
    wasm_val_vec_t results = WASM_EMPTY_VEC;
    if (wasm_func_call(exportedFuncMap["__wasm_call_ctors"], &args, &results)) {
        printf("> Error calling function!\n");
        return;
    }
    for (std::string const modName : resourceStorage->modNames) {
        std::string const stdFuncName = "init_mod_" + modName;
        if (wasm_func_call(exportedFuncMap[stdFuncName], &args, &results)) {
            printf("> Error calling function!\n");
            return;
        }
    }
    nrStartFunc = exportedFuncMap["wr_start"];
    assert(nrStartFunc != nullptr);

    isInitialized = true;
}
void WasmCApiModsRuntime::start() {
    wasm_val_vec_t const args = WASM_EMPTY_VEC;
    wasm_val_vec_t results = WASM_EMPTY_VEC;
    if (wasm_trap_t *trap = wasm_func_call(nrStartFunc, &args, &results)) {
        wasm_message_t message;
        wasm_trap_message(trap, &message);
        if (std::string(message.data, message.size - 1) != procExitMessage)
            printf("> Error calling function!\n");
        return;
    }
}

bool WasmCApiModsRuntime::getVMData(void *outPtr, uint64_t offset,
                                    int32_t size) {
    byte_t *data = wasm_memory_data(memory);
    size_t const vmSize = wasm_memory_data_size(memory);
    if (offset < 0 || offset + size >= vmSize)
        return false;
    memcpy(outPtr, data + offset, size);
    return true;
}
bool WasmCApiModsRuntime::setVMData(const void *inPtr, uint64_t offset,
                                    int32_t size) {
    byte_t *data = wasm_memory_data(memory);
    size_t const vmSize = wasm_memory_data_size(memory);
    if (offset < 0 || offset + size >= vmSize)
        return false;
    memcpy(data + offset, inPtr, size);
    return true;
}

size_t WasmCApiModsRuntime::vmSize() {
    return wasm_memory_data_size(memory);
}

size_t WasmCApiModsRuntime::voidptrSize() {
    return 4;
};

WasmCApiModsRuntime::~WasmCApiModsRuntime() {
    wasm_module_delete(module);
    wasm_instance_delete(instance);

    //
    wasm_extern_vec_delete(exports);
    delete exports;
    wasm_store_delete(store);
    wasm_engine_delete(engine);
}
} // namespace wasm_c_api
} // namespace runtimes
} // namespace webrogue
