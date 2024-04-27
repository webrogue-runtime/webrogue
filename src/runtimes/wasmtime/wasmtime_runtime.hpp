#pragma once

#include "../../core/ModsRuntime.hpp"
#include "wasmtime.h"
#include <string>

namespace webrogue {
namespace runtimes {
namespace wasmtime {
class WasmtimeModsRuntime : public webrogue::core::ModsRuntime {
public:
    WasmtimeModsRuntime(webrogue::core::ConsoleStream *wrout,
                        webrogue::core::ConsoleStream *wrerr,
                        webrogue::core::ResourceStorage *resourceStorage,
                        webrogue::core::Config *config);

    wasmtime_instance_t instance;
    wasm_config_t *wasmtimeConfig;
    wasm_engine_t *engine;
    wasmtime_store_t *store;
    wasmtime_context_t *context;
    wasmtime_module_t *module = NULL;
    wasmtime_memory_t memory;

    bool callExportedFunc(std::string funcName);

    void initMods() override;
    void start() override;
    bool getVMData(void *outPtr, uint64_t offset, int32_t size) override;
    bool setVMData(const void *inPtr, uint64_t offset, int32_t size) override;
    size_t vmSize() override;
    size_t voidptrSize() override;
    ~WasmtimeModsRuntime() override;
};
} // namespace wasmtime
} // namespace runtimes
} // namespace webrogue
