#include "EventManager.hpp"

namespace webrogue {
namespace core {
void EventManager::poll(uint32_t timeoutMs) {
    for (auto &pollable : pollables)
        pollable->poll(*this);
}

void EventManager::addEvent(webrogue_event event) {
    eventBuffer.push_back(event);
}

void EventManager::registerPollable(Pollable *pollable) {
    pollables.push_back(pollable);
}

std::vector<webrogue_event> const &EventManager::getEvents() {
    return eventBuffer;
}

void EventManager::clearEvents() {
    eventBuffer.clear();
}
} // namespace core
} // namespace webrogue
