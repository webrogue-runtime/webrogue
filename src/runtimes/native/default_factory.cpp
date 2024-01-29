#include "../../core/ModsRuntime.hpp"
#include "native_runtime.hpp"
#include "shared_api_object.hpp"

namespace webrogue {
namespace runtimes {
std::shared_ptr<webrogue::core::ModsRuntime> MAKE_DEFAULT_RUNTIME_EXPORT
makeDefaultRuntime(webrogue::core::ConsoleStream *wrout,
                   webrogue::core::ConsoleStream *wrerr,
                   webrogue::core::ResourceStorage *resourceStorage,
                   webrogue::core::Config *config) {
    auto result = std::make_shared<native::NativeModsRuntime>(
        wrout, wrerr, resourceStorage, config);
    webrogue::runtimes::native::sharedApiObject = &result->apiObject;
    return result;
}
} // namespace runtimes
} // namespace webrogue
