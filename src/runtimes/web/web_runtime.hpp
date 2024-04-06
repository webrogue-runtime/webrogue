#pragma once

#include "../../core/CompactLinking.hpp"
#include "../../core/ModsRuntime.hpp"
#include <string>

namespace webrogue {
namespace runtimes {
namespace web {
class WebModsRuntime : public webrogue::core::ModsRuntime {
public:
    WebModsRuntime(webrogue::core::ResourceStorage *resourceStorage,
                   webrogue::core::Config const *config);
    std::shared_ptr<core::wasm_binnary> linkedWasm;

    void initMods() override;
    void start() override;
    bool getVMData(void *outPtr, uint64_t offset, int32_t size) override;
    bool setVMData(const void *inPtr, uint64_t offset, int32_t size) override;
    size_t vmSize() override;
    bool execAsyncFunc(std::string funcName);
    size_t voidptrSize() override;
    ~WebModsRuntime() override;
};
} // namespace web
} // namespace runtimes
} // namespace webrogue
