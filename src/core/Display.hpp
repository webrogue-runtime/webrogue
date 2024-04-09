#pragma once

#include "Pollable.hpp"
#include "Terminal.hpp"
#include <cstdint>
#include <memory>

namespace webrogue {
namespace core {
class Display : public Pollable {
public:
    Display();
    std::unique_ptr<Terminal> terminal = nullptr;
    virtual ~Display();
};
} // namespace core
} // namespace webrogue
