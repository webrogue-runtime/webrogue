#pragma once

#include <cstddef>
namespace webrogue {
namespace core {
class Terminal {
public:
    virtual ~Terminal();
    virtual void writeStdout(void const *data, size_t size) = 0;
};
} // namespace core
} // namespace webrogue
