#pragma once

#include "../../external/libtsm/src/tsm/libtsm.h"
#include "Terminal.hpp"
#include <cstddef>
#include <cstdint>

namespace webrogue {
namespace core {
class TSMTerminal : public Terminal {
public:
    TSMTerminal();
    ~TSMTerminal() override;
    void writeStdout(void const *data, size_t size) override;
    virtual void drawGlyph(int x, int y, uint32_t glyph) = 0;
    virtual int getWidth() = 0;
    virtual int getHeight() = 0;

    tsm_screen *screen;
    tsm_vte *vte;
};
} // namespace core
} // namespace webrogue
