#include "../../core/ModsRuntime.hpp"
#include "web_runtime.hpp"

namespace webrogue {
namespace runtimes {
std::shared_ptr<webrogue::core::ModsRuntime> MAKE_DEFAULT_RUNTIME_EXPORT
makeDefaultRuntime(webrogue::core::ConsoleStream *wrout,
                   webrogue::core::ConsoleStream *wrerr,
                   webrogue::core::ResourceStorage *resourceStorage,
                   webrogue::core::Config *config) {
    return std::make_shared<web::WebModsRuntime>(wrout, wrerr, resourceStorage,
                                                 config);
}
} // namespace runtimes
} // namespace webrogue