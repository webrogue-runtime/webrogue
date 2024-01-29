#pragma once

#include "../../../external/wamr/core/iwasm/include/wasm_export.h"
#include "../../core/Config.hpp"
#include "../../core/ModsRuntime.hpp"

namespace webrogue {
namespace runtimes {
namespace wamr {
class WamrModsRuntime : public webrogue::core::ModsRuntime {
public:
    WamrModsRuntime(webrogue::core::ConsoleStream *wrout,
                    webrogue::core::ConsoleStream *wrerr,
                    webrogue::core::ResourceStorage *resourceStorage,
                    webrogue::core::Config *config);
    const uint32_t stackSize = 64 * 1024, heapSize = 64 * 1024;

    bool initializedWasmRuntime = false;
    wasm_module_t module = nullptr;
    wasm_module_inst_t moduleInst = nullptr;
    wasm_exec_env_t execEnv = nullptr;
    wasm_function_inst_t startFunc = nullptr;

    void initMods() override;
    void start() override;
    bool getVMData(void *outPtr, uint64_t offset, int32_t size) override;
    bool setVMData(const void *inPtr, uint64_t offset, int32_t size) override;
    size_t voidptrSize() override;
    ~WamrModsRuntime() override;
};
} // namespace wamr
} // namespace runtimes
} // namespace webrogue
