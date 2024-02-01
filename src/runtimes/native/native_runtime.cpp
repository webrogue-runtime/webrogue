#include "native_runtime.hpp"

#include "../../common/stringize.hpp"
#include "../../core/ConsoleStream.hpp"
#include "wr_api_native_glue.hpp"
#include <cstring>
#include <vector>

extern "C" void wr_start();

// #define WR_API_FUNCTION(RET_TYPE, NAME, ARGS) extern "C" RET_TYPE NAME ARGS;
// #include "../../../mods/core/include/common/wr_api_functions.def"
// #undef WR_API_FUNCTION

#define mod_to_embed(name) extern "C" void init_mod_##name();
#include stringize(WEBROGUE_MOD_LIST_HEADER)
#undef mod_to_embed

namespace webrogue {
namespace runtimes {
namespace native {

NativeModsRuntime::NativeModsRuntime(
    webrogue::core::ConsoleStream *wrout, webrogue::core::ConsoleStream *wrerr,
    webrogue::core::ResourceStorage *resourceStorage,
    webrogue::core::Config *config)
    : ModsRuntime(wrout, wrerr, resourceStorage, config) {
}

void NativeModsRuntime::initMods() {
    // TODO remove
    //  needed to mark api functions as used to prevent strange linking bug
    initWrNativeApi();

    *wrout << "initialization started\n";
#define mod_to_embed(name) init_mod_##name();
#include stringize(WEBROGUE_MOD_LIST_HEADER)
#undef mod_to_embed
    *wrout << "initialization finished\n";
    isInitialized = true;
}
void NativeModsRuntime::start() {
    wr_start();
}

bool NativeModsRuntime::getVMData(void *outPtr, uint64_t offset, int32_t size) {
    memcpy(outPtr, (void *)offset, size);
    return true;
}
bool NativeModsRuntime::setVMData(const void *inPtr, uint64_t offset,
                                  int32_t size) {

    memcpy((void *)offset, inPtr, size);
    return true;
}
size_t NativeModsRuntime::vmSize() {
    return -1;
}
size_t NativeModsRuntime::voidptrSize() {
    return sizeof(void *);
};
} // namespace native
} // namespace runtimes
} // namespace webrogue
