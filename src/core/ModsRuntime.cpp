#include "ModsRuntime.hpp"

#include "ApiObject.hpp"
#include "WASIObject.hpp"
#include <chrono>
namespace webrogue {
namespace core {
ModsRuntime::ModsRuntime(ConsoleStream *wrout, ConsoleStream *wrerr,
                         ResourceStorage *resourceStorage, Config *config)
    : wrout(wrout), wrerr(wrerr), resourceStorage(resourceStorage),
      config(config),
#ifndef WEBROGUE_NO_WASI
      wasiObject(this, resourceStorage, config),
#endif
      apiObject(this, config) {
}

ModsRuntime::~ModsRuntime() {
}
void ModsRuntime::interrupt() {
    auto now = std::chrono::system_clock::now();
    static auto lastInterrupt = now;

    if (std::chrono::duration_cast<std::chrono::milliseconds>(now -
                                                              lastInterrupt)
            .count() > 100) {
        apiObject.output->lazyEnd();
        apiObject.output->beginFrame();
        lastInterrupt = std::chrono::system_clock::now();
    }
}

bool ModsRuntime::isVMRangeValid(uint64_t offset, int32_t size) {
    return offset + size < vmSize();
}

void ModsRuntime::onFrameEnd() {
    config->onFrameEnd();
}
} // namespace core
} // namespace webrogue
