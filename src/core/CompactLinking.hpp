#pragma once

#include "Config.hpp"
#include "ConsoleStream.hpp"
#include "ModsRuntime.hpp"
#include "ResourceStorage.hpp"
#include <cstdint>
#include <memory>
#include <vector>

namespace webrogue {
namespace core {
typedef std::vector<uint8_t> wasm_binnary;

std::shared_ptr<std::vector<uint8_t>> getCompactlyLinkedBinaries(
    ModsRuntime *runtime, ResourceStorage *resourceStorage,
    Config const *config, std::function<void()> interrupt);

} // namespace core
} // namespace webrogue
