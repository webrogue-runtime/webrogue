#pragma once

#include "../../core/ModsRuntime.hpp"

extern "C" struct wasm_module_t;
extern "C" struct wasm_instance_t;
extern "C" struct wasm_engine_t;
extern "C" struct wasm_store_t;
extern "C" struct wasm_extern_vec_t;
extern "C" struct wasm_memory_t;
extern "C" struct wasm_func_t;
namespace webrogue {
namespace runtimes {
namespace wasm_c_api {
class WasmCApiModsRuntime : public webrogue::core::ModsRuntime {
public:
    WasmCApiModsRuntime(webrogue::core::ResourceStorage *resourceStorage,
                        webrogue::core::Config const *config);
    wasm_module_t *module = nullptr;
    wasm_instance_t *instance = nullptr;
    wasm_engine_t *engine = nullptr;
    wasm_store_t *store = nullptr;
    wasm_extern_vec_t *exports = nullptr;
    wasm_memory_t *memory = nullptr;
    const wasm_func_t *nrStartFunc = nullptr;
    void initMods() override;
    void start() override;
    bool getVMData(void *outPtr, uint64_t offset, int32_t size) override;
    bool setVMData(const void *inPtr, uint64_t offset, int32_t size) override;
    size_t vmSize() override;
    size_t voidptrSize() override;
    ~WasmCApiModsRuntime() override;
};
} // namespace wasm_c_api
} // namespace runtimes
} // namespace webrogue
