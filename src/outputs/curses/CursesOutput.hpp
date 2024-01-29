#pragma once

#include "../../core/Output.hpp"

namespace webrogue {
namespace outputs {
namespace curses {
class CursesOutput : public webrogue::core::Output {
public:
    CursesOutput();
    ~CursesOutput() override;

protected:
    int dx, dy;
    void pollEvent(int milliseconds);

    bool isKeyboardAvailable() override;

    void startColor() override;

    int32_t getColorPairsCount() override;

    int32_t getColorsCount() override;

    void setColor(int32_t color, int32_t r, int32_t g, int32_t b) override;

    void setColorPair(int32_t colorPair, int32_t fg, int32_t bg) override;

    void onBegin() override;

    void onEnd() override;

    void onBeginFrame() override;

    void onEndFrame() override;
};
} // namespace curses
} // namespace outputs
} // namespace webrogue
