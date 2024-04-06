#pragma once

#include "Config.hpp"
#include "ModsRuntime.hpp"
#include <memory>

namespace webrogue {
namespace core {
int webrogueMain(webrogue::core::runtime_maker_t runtimeMaker,
                 const webrogue::core::Config config);
} // namespace core
} // namespace webrogue
