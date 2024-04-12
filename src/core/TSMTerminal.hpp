#pragma once

#include "../../external/libtsm/src/tsm/libtsm.h"
#include "Terminal.hpp"
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <vector>

namespace webrogue {
namespace core {
class TSMTerminal : public Terminal {
public:
    TSMTerminal();
    ~TSMTerminal() override;
    void feedText(void const *data, size_t size);
    void writeStdout(void const *data, size_t size) override;
    void writeStdin(void const *data, size_t size) override;
    void poll(EventManager &eventManager) override;

    struct GlyphColor {
        uint8_t foregroundRed;
        uint8_t foregroundGreen;
        uint8_t foregroundBlue;
        uint8_t backgroundRed;
        uint8_t backgroundGreen;
        uint8_t backgroundBlue;
        uint16_t foregroundCode;
        uint16_t backgroundCode;
    };
    virtual void drawGlyph(int x, int y, uint32_t glyph, GlyphColor color) = 0;
    void bufferedDraw();
    virtual void draw();
    virtual int getWidth() = 0;
    virtual int getHeight() = 0;

    bool ignoreStdin = false;

protected:
    std::chrono::steady_clock::time_point lastDrawTimePoint =
        std::chrono::steady_clock::now();
    tsm_screen *screen;
    tsm_vte *vte;

    std::vector<uint8_t> stdinBuffer;
};
} // namespace core
} // namespace webrogue
