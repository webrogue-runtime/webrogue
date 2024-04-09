#pragma once

#include "../../mods/core/include/common/events.h"
#include "Pollable.hpp"
#include <cstddef>
#include <cstdint>
#include <vector>

namespace webrogue {
namespace core {
class EventManager {
public:
    void poll(uint32_t timeoutMs);
    void addEvent(webrogue_event event);
    void registerPollable(Pollable *pollable);
    std::vector<webrogue_event> const &getEvents();
    void clearEvents();

private:
    std::vector<webrogue_event> eventBuffer;
    std::vector<Pollable *> pollables;
};
} // namespace core
} // namespace webrogue
