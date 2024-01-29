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
            .count() > 10) {
        lastInterrupt = now;
        apiObject.output->lazyEnd();
        apiObject.output->beginFrame();
    }
}

void ModsRuntime::onFrameEnd() {

#ifndef WEBROGUE_NO_WASI
    wasiObject.vfs.commit();
#endif
    config->onFrameEnd();
}
} // namespace core
} // namespace webrogue
