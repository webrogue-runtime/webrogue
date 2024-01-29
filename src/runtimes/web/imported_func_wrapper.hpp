#pragma once

#include "../../core/ApiObject.hpp"
#include "../../core/WASIObject.hpp"
#include <string>

namespace webrogue {
namespace runtimes {
namespace web {
void initWrapper(webrogue::core::ModsRuntime *runtime);
void callImportedFunc(int funcId);
} // namespace web
} // namespace runtimes
} // namespace webrogue
