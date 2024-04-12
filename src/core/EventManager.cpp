#include "EventManager.hpp"
#include <cstring>

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
    hasStdinEvent = false;
}
void EventManager::writeStdin(const std::vector<uint8_t> stdinBuffer) {
    if (!hasStdinEvent) {
        addEvent({webrogue_event_type::Stdin});
        hasStdinEvent = true;
    }
    this->stdinBuffer.resize(this->stdinBuffer.size() + stdinBuffer.size());
    memcpy(this->stdinBuffer.data() + this->stdinBuffer.size() -
               stdinBuffer.size(),
           stdinBuffer.data(), stdinBuffer.size());
}
} // namespace core
} // namespace webrogue
