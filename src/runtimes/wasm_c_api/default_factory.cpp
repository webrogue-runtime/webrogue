#include "../../core/ModsRuntime.hpp"
#include "wasm_c_api_runtime.hpp"

namespace webrogue {
namespace runtimes {
std::shared_ptr<webrogue::core::ModsRuntime> MAKE_DEFAULT_RUNTIME_EXPORT
makeDefaultRuntime(webrogue::core::ConsoleStream *wrout,
                   webrogue::core::ConsoleStream *wrerr,
                   webrogue::core::ResourceStorage *resourceStorage,
                   webrogue::core::Config *config) {
    auto result = std::make_shared<wasm_c_api::WasmCApiModsRuntime>(
        wrout, wrerr, resourceStorage, config);
    return result;
}
} // namespace runtimes
} // namespace webrogue
