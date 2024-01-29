#pragma once

#include "Config.hpp"
#include "ModsRuntime.hpp"
#include "Output.hpp"
#include <memory>

namespace webrogue {
namespace core {
int webrogueMain(std::shared_ptr<webrogue::core::Output> output,
                 webrogue::core::runtime_maker_t runtimeMaker,
                 webrogue::core::Config *config);
} // namespace core
} // namespace webrogue
