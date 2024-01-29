#pragma once

#include "../../../external/wasm3/source/wasm3.h"
#include "../../core/CompactLinking.hpp"
#include "../../core/ModsRuntime.hpp"

namespace webrogue {
namespace runtimes {
namespace m3 {
class M3ModsRuntime : public webrogue::core::ModsRuntime {
public:
    M3ModsRuntime(webrogue::core::ConsoleStream *wrout,
                  webrogue::core::ConsoleStream *wrerr,
                  webrogue::core::ResourceStorage *resourceStorage,
                  webrogue::core::Config *config);
    std::shared_ptr<core::wasm_binnary> linkedWasm;
    IM3Environment modsEnvironment = nullptr;
    IM3Runtime modsRuntime = nullptr;
    IM3Module modsModule = nullptr;
    IM3Function startFunction = nullptr;

    void initMods() override;
    void start() override;
    bool getVMData(void *outPtr, uint64_t offset, int32_t size) override;
    bool setVMData(const void *inPtr, uint64_t offset, int32_t size) override;
    size_t voidptrSize() override;
    ~M3ModsRuntime() override;
};
} // namespace m3
} // namespace runtimes
} // namespace webrogue
