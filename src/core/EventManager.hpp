#pragma once

#include <cstdint>

namespace webrogue {
namespace core {
class EventManager {
public:
    uint32_t poll(uint32_t timeoutMs);
};
} // namespace core
} // namespace webrogue
