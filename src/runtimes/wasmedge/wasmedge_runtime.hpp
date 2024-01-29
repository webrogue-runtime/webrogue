#pragma once

#include "../../../external/wasmedge/include/api/wasmedge/wasmedge.h"
#include "../../core/ModsRuntime.hpp"

namespace webrogue {
namespace runtimes {
namespace wasmedge {
class WasmEdgeModsRuntime : public webrogue::core::ModsRuntime {
public:
    WasmEdge_VMContext *vmCxt;
    WasmEdge_ConfigureContext *confCxt;
    WasmEdgeModsRuntime(webrogue::core::ConsoleStream *wrout,
                        webrogue::core::ConsoleStream *wrerr,
                        webrogue::core::ResourceStorage *resourceStorage,
                        webrogue::core::Config *config);
    void initMods() override;
    void start() override;
    bool getVMData(void *outPtr, uint64_t offset, int32_t size) override;
    bool setVMData(const void *inPtr, uint64_t offset, int32_t size) override;
    size_t voidptrSize() override;
    ~WasmEdgeModsRuntime() override;
};
} // namespace wasmedge
} // namespace runtimes
} // namespace webrogue
