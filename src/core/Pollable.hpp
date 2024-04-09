#pragma once

#include <cstdint>

namespace webrogue {
namespace core {
class EventManager;
class Pollable {
public:
    virtual void poll(EventManager &eventManager) = 0;
};
} // namespace core
} // namespace webrogue
