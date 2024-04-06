#pragma once

#include "ApiObject.hpp"
#include "Config.hpp"
#include "ConsoleStream.hpp"
#include "Display.hpp"
#include "EventManager.hpp"
#include "ResourceStorage.hpp"
#include "WASIObject.hpp"
#include <cstddef>
#include <cstdint>
#include <functional>
#include <memory>

namespace webrogue {
namespace core {
class ModsRuntime {
public:
    ModsRuntime(webrogue::core::ResourceStorage *,
                webrogue::core::Config const *);
    ApiObject apiObject;
#ifndef WEBROGUE_NO_WASI
    WASIObject wasiObject;
#endif
    EventManager eventManager;
    std::shared_ptr<Display> display;
    ResourceStorage *resourceStorage;
    Config const *config;
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
    virtual size_t vmSize() = 0;

    template <typename T> inline void setVMInt(uint64_t offset, T value) {
        T swapped = byteswap<T>(value);
        setVMData(&swapped, offset, sizeof(T));
    }

    template <typename T> inline T getVMInt(uint64_t offset) {
        T swapped;
        getVMData(&swapped, offset, sizeof(T));
        return byteswap<T>(swapped);
    }

    inline void setVMDataZeros(uint64_t offset, int32_t size) {
        void *data = calloc(0, size);
        setVMData(data, offset, size);
        free(data);
    }

    bool isVMRangeValid(uint64_t offset, int32_t size);
    virtual ~ModsRuntime();

    void interrupt();
};

typedef std::function<std::shared_ptr<ModsRuntime>(ResourceStorage *,
                                                   Config const *)>
    runtime_maker_t;

} // namespace core
} // namespace webrogue

namespace webrogue {
namespace runtimes {

#ifndef MAKE_DEFAULT_RUNTIME_EXPORT
#define MAKE_DEFAULT_RUNTIME_EXPORT
#endif

std::shared_ptr<webrogue::core::ModsRuntime> MAKE_DEFAULT_RUNTIME_EXPORT
makeDefaultRuntime(webrogue::core::ResourceStorage *resourceStorage,
                   webrogue::core::Config const *config);
} // namespace runtimes
} // namespace webrogue
