#include "wasmtime_runtime.hpp"
#include "../../common/stringize.hpp"
#include "../../core/ApiObject.hpp"
#include "../../core/CompactLinking.hpp"
#include "../../core/ConsoleStream.hpp"
#include "wasmtime/config.h"
#include "wasmtime/module.h"
#include "wasmtime/trap.h"
#include "wasmtime_templates.hpp"
#include <cassert>
#include <cstddef>
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
namespace wasmtime {

static const std::string procExitMessage = "proc_exit";

static wasm_trap_t *procExitFunc(void *env, wasmtime_caller_t *caller,
                                 const wasmtime_val_t *args, size_t nargs,
                                 wasmtime_val_t *results, size_t nresults) {
    auto *runtime =
        reinterpret_cast<webrogue::runtimes::wasmtime::WasmtimeModsRuntime *>(
            env);
    runtime->procExit = true;
    runtime->procExitCode = args[0].of.f32;
    return wasmtime_trap_new(procExitMessage.data(), procExitMessage.size());
}

static void exit_with_error(const char *message, wasmtime_error_t *error,
                            wasm_trap_t *trap) {
    fprintf(stderr, "error: %s\n", message);
    wasm_byte_vec_t error_message;
    if (error != NULL) {
        wasmtime_error_message(error, &error_message);
        wasmtime_error_delete(error);
    } else if (trap != NULL) {
        wasm_trap_message(trap, &error_message);
        wasm_trap_delete(trap);
    } else {
        fprintf(stderr, "%s", message);
        exit(1);
    }
    fprintf(stderr, "%.*s\n", (int)error_message.size, error_message.data);
    wasm_byte_vec_delete(&error_message);
    exit(1);
}

WasmtimeModsRuntime::WasmtimeModsRuntime(
    webrogue::core::ConsoleStream *wrout, webrogue::core::ConsoleStream *wrerr,
    webrogue::core::ResourceStorage *resourceStorage,
    webrogue::core::Config *config)
    : ModsRuntime(wrout, wrerr, resourceStorage, config) {
}

void WasmtimeModsRuntime::initMods() {
    auto linkResult = core::getCompactlyLinkedBinaries(
        this, resourceStorage, config,
        [this]() {
            interrupt();
        },
        wrout, wrerr);

    wasmtimeConfig = wasm_config_new();
    wasmtime_config_debug_info_set(wasmtimeConfig, true);
    wasmtime_config_cache_config_load(wasmtimeConfig, NULL);
    engine = wasm_engine_new_with_config(wasmtimeConfig);
    assert(engine != NULL);

    store = wasmtime_store_new(engine, NULL, NULL);
    assert(store != NULL);
    context = wasmtime_store_context(store);
    wasm_byte_vec_t wasmData;
    wasm_byte_vec_new_uninitialized(&wasmData, linkResult->size());
    memcpy(wasmData.data, linkResult->data(), linkResult->size());
    wasmtime_error_t *error;
    error = wasmtime_module_new(engine, (uint8_t *)wasmData.data, wasmData.size,
                                &module);
    wasm_byte_vec_delete(&wasmData);

    if (error != NULL)
        exit_with_error("failed to compile module", error, NULL);

    std::map<std::string, wasmtime_extern_t> importMap;

#define WR_API_FUNCTION(RET_TYPE, NAME, ARGS)                                  \
    FuncLinker<decltype(&core::ApiObject::NAME),                               \
               &core::ApiObject::NAME>::link("webrogue", stringize(NAME),      \
                                             context, this, importMap);
#include "../../../mods/core/include/common/wr_api_functions.def"
#undef WR_API_FUNCTION

#define WASI_FUNCTION(RET_TYPE, NAME, ARGS)                                    \
    FuncLinker<decltype(&core::WASIObject::NAME),                              \
               &core::WASIObject::NAME>::link("wasi_snapshot_preview1",        \
                                              stringize(NAME), context, this,  \
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
        wasmtime_func_t wrappedWasmFuncType;

        wasmtime_func_new(context, wrappedFuncType, procExitFunc, this, NULL,
                          &wrappedWasmFuncType);

        wasm_functype_delete(wrappedFuncType);

        wasmtime_extern_t import;
        import.kind = WASMTIME_EXTERN_FUNC;
        import.of.func = wrappedWasmFuncType;

        importMap["wasi_snapshot_preview1.proc_exit"] = import;
    }

    wasm_importtype_vec_t expectedImports;
    wasmtime_module_imports(module, &expectedImports);

    std::vector<wasmtime_extern_t> externs;
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

    wasm_trap_t *trap = NULL;
    error = wasmtime_instance_new(context, module, externs.data(),
                                  externs.size(), &instance, &trap);

    wasmtime_extern_t item;
    bool const ok = wasmtime_instance_export_get(context, &instance, "memory",
                                                 strlen("memory"), &item);
    assert(ok && item.kind == WASMTIME_EXTERN_MEMORY);
    memory = item.of.memory;

    if (error != NULL || trap != NULL)
        exit_with_error("failed to instantiate", error, trap);

    *wrout << "initializing mods...\n";
    callExportedFunc("__wasm_call_ctors");
    for (std::string const modName : resourceStorage->modNames)
        callExportedFunc("init_mod_" + modName);

    isInitialized = true;
}

bool WasmtimeModsRuntime::callExportedFunc(std::string funcName) {

    wasmtime_extern_t run;
    bool const ok = wasmtime_instance_export_get(
        context, &instance, funcName.data(), funcName.size(), &run);
    assert(ok);
    assert(run.kind == WASMTIME_EXTERN_FUNC);

    wasm_trap_t *trap = NULL;
    wasmtime_error_t *error =
        wasmtime_func_call(context, &run.of.func, NULL, 0, NULL, 0, &trap);
    if (error != NULL)
        exit_with_error("failed to call function", error, trap);

    return true;
}

void WasmtimeModsRuntime::start() {
    callExportedFunc("wr_start");
}

bool WasmtimeModsRuntime::getVMData(void *outPtr, uint64_t offset,
                                    int32_t size) {
    memcpy(outPtr, wasmtime_memory_data(context, &memory) + offset, size);
    return true;
}
bool WasmtimeModsRuntime::setVMData(const void *inPtr, uint64_t offset,
                                    int32_t size) {
    memcpy(wasmtime_memory_data(context, &memory) + offset, inPtr, size);
    return true;
}

size_t WasmtimeModsRuntime::vmSize() {
    return wasmtime_memory_data_size(context, &memory);
}

size_t WasmtimeModsRuntime::voidptrSize() {
    return 4;
};

WasmtimeModsRuntime::~WasmtimeModsRuntime() {
    wasmtime_module_delete(module);
    wasmtime_store_delete(store);
    wasm_engine_delete(engine);
}
} // namespace wasmtime
} // namespace runtimes
} // namespace webrogue
