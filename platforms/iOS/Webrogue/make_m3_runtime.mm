#include "make_m3_runtime.h"
#include "../../../src/runtimes/m3/m3_runtime.hpp"

namespace webrogue {
namespace runtimes {
std::shared_ptr<webrogue::core::ModsRuntime> MAKE_DEFAULT_RUNTIME_EXPORT
makeM3Runtime(webrogue::core::ConsoleStream *wrout,
                   webrogue::core::ConsoleStream *wrerr,
                   webrogue::core::ResourceStorage *resourceStorage,
                   webrogue::core::Config *config) {
    auto result = std::make_shared<m3::M3ModsRuntime>(
        wrout, wrerr, resourceStorage, config);
    return result;
}
} // namespace runtimes
} // namespace webrogue
