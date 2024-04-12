#pragma once

#include "Pollable.hpp"
#include <cstddef>

namespace webrogue {
namespace core {
class Terminal : public Pollable {
public:
    virtual ~Terminal();
    virtual void writeStdout(void const *data, size_t size) = 0;
    virtual void writeStdin(void const *data, size_t size) = 0;
};
} // namespace core
} // namespace webrogue
