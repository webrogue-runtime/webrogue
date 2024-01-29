#include "make_wasm_c_api_runtime.h"
#include "../../../src/runtimes/wasm_c_api/wasm_c_api_runtime.hpp"

namespace webrogue {
namespace runtimes {
std::shared_ptr<webrogue::core::ModsRuntime> 
makeWasmCApiRuntime(webrogue::core::ConsoleStream *wrout,
                   webrogue::core::ConsoleStream *wrerr,
                   webrogue::core::ResourceStorage *resourceStorage,
                   webrogue::core::Config *config) {
    auto result = std::make_shared<wasm_c_api::WasmCApiModsRuntime>(
        wrout, wrerr, resourceStorage, config);
    return result;
}
} // namespace runtimes
} // namespace webrogue
