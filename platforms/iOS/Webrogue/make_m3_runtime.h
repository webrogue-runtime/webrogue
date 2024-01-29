#pragma once

#include "../../../src/core/ModsRuntime.hpp"

namespace webrogue {
namespace runtimes {
std::shared_ptr<webrogue::core::ModsRuntime>
makeM3Runtime(webrogue::core::ConsoleStream *wrout,
                webrogue::core::ConsoleStream *wrerr,
                webrogue::core::ResourceStorage *resourceStorage,
                webrogue::core::Config *config);
} // namespace runtimes
} // namespace webrogue
