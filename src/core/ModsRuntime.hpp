#pragma once

#include "ApiObject.hpp"
#include "Config.hpp"
#include "ConsoleStream.hpp"
#include "ResourceStorage.hpp"
#include "WASIObject.hpp"
#include <cstddef>
#include <functional>
#include <memory>

namespace webrogue {
namespace core {
class Linker;
class ModsRuntime {
public:
    ModsRuntime(ConsoleStream *wrout, ConsoleStream *wrerr,
                ResourceStorage *resourceStorage, Config *config);
    ApiObject apiObject;
#ifndef WEBROGUE_NO_WASI
    WASIObject wasiObject;
#endif
    Linker *linker;
    ConsoleStream *wrout;
    ConsoleStream *wrerr;
    ResourceStorage *resourceStorage;
    Config *config;
    const void *vmContext = nullptr;

    bool procExit = false;
    int32_t procExitCode;

    bool isInitialized = false;

    virtual void initMods() = 0;
    virtual void start() = 0;
    virtual bool getVMData(void *outPtr, uint64_t offset, int32_t size) = 0;
    virtual bool setVMData(const void *in_ptr, uint64_t offset,
                           int32_t size) = 0;
    virtual size_t voidptrSize() = 0;
    virtual ~ModsRuntime();

    void interrupt();

    void onFrameEnd();
};

typedef std::function<std::shared_ptr<ModsRuntime>(
    ConsoleStream *, ConsoleStream *, ResourceStorage *, Config *)>
    runtime_maker_t;

} // namespace core
} // namespace webrogue

namespace webrogue {
namespace runtimes {

#ifndef MAKE_DEFAULT_RUNTIME_EXPORT
#define MAKE_DEFAULT_RUNTIME_EXPORT
#endif

std::shared_ptr<webrogue::core::ModsRuntime> MAKE_DEFAULT_RUNTIME_EXPORT
makeDefaultRuntime(webrogue::core::ConsoleStream *wrout,
                   webrogue::core::ConsoleStream *wrerr,
                   webrogue::core::ResourceStorage *resourceStorage,
                   webrogue::core::Config *config);
} // namespace runtimes
} // namespace webrogue
