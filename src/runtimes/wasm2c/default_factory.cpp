#include "../../core/ModsRuntime.hpp"
#include "wasm2c_runtime.hpp"

namespace webrogue {
namespace runtimes {

std::shared_ptr<webrogue::core::ModsRuntime> MAKE_DEFAULT_RUNTIME_EXPORT
makeDefaultRuntime(webrogue::core::ResourceStorage *resourceStorage,
                   webrogue::core::Config const *config) {
    return std::make_shared<wasm2c::Wasm2cModsRuntime>(resourceStorage, config);
}
} // namespace runtimes
} // namespace webrogue
