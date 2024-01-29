#pragma once

#include "../../common/stringize.hpp"
#include "../../core/Config.hpp"
#include "../../core/ModsRuntime.hpp"

#include stringize(LINKED_HEADER)

namespace webrogue {
namespace runtimes {
namespace wasm2c {
class Wasm2cModsRuntime : public webrogue::core::ModsRuntime {
public:
    Wasm2cModsRuntime(webrogue::core::ConsoleStream *wrout,
                      webrogue::core::ConsoleStream *wrerr,
                      webrogue::core::ResourceStorage *resourceStorage,
                      webrogue::core::Config *config);

    w2c_linked linked;
    void initMods() override;
    void start() override;
    bool getVMData(void *outPtr, uint64_t offset, int32_t size) override;
    bool setVMData(const void *inPtr, uint64_t offset, int32_t size) override;
    size_t voidptrSize() override;
};
} // namespace wasm2c
} // namespace runtimes
} // namespace webrogue
