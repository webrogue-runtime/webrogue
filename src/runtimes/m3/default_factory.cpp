#include "../../core/ModsRuntime.hpp"
#include "m3_runtime.hpp"

namespace webrogue {
namespace runtimes {
std::shared_ptr<webrogue::core::ModsRuntime> MAKE_DEFAULT_RUNTIME_EXPORT
makeDefaultRuntime(webrogue::core::ResourceStorage *resourceStorage,
                   webrogue::core::Config const *config) {
    return std::make_shared<m3::M3ModsRuntime>(resourceStorage, config);
}
} // namespace runtimes
} // namespace webrogue
