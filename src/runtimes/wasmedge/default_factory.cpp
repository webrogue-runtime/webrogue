#include "../../../external/wasmedge/include/api/wasmedge/wasmedge.h"
#include "../../core/ModsRuntime.hpp"
#include "wasmedge_runtime.hpp"

namespace webrogue {
namespace runtimes {
std::shared_ptr<webrogue::core::ModsRuntime> MAKE_DEFAULT_RUNTIME_EXPORT
makeDefaultRuntime(webrogue::core::ConsoleStream *wrout,
                   webrogue::core::ConsoleStream *wrerr,
                   webrogue::core::ResourceStorage *resourceStorage,
                   webrogue::core::Config *config) {
    auto result = std::make_shared<wasmedge::WasmEdgeModsRuntime>(
        wrout, wrerr, resourceStorage, config);
    return result;
}
} // namespace runtimes
} // namespace webrogue
