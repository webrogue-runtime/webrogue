#pragma once

#include "../../mods/core/include/common/colors.h"
#include "../../mods/core/include/common/events.h"
#include "Buffer2d.hpp"
#include "Vec2.hpp"
#include <chrono>
#include <cstdint>
#include <list>
#include <stack>

namespace webrogue {
namespace core {
class Output {
protected:
    const int32_t maxWidth = 200;
    const int32_t maxHeight = 50;
    Vec2Int storedSize = Vec2Int(0, 0);
    Buffer2d<wr_glyph> renderBuffer;
    bool isRendering = false;
    bool isRenderingFrame = false;
    std::stack<webrogue_event> events;
    std::chrono::steady_clock::time_point deadline;
    bool hasDeadline = false;

public:
    virtual bool isKeyboardAvailable() = 0;

    virtual void startColor() = 0;

    virtual int32_t getColorPairsCount() = 0;

    virtual int32_t getColorsCount() = 0;

    virtual void setColor(int32_t color, int32_t r, int32_t g, int32_t b) = 0;

    virtual void setColorPair(int32_t colorPair, int32_t fg, int32_t bg) = 0;

    void begin();

    void beginFrame();

    void endFrame();

    void end();

    Vec2Int size();

    virtual ~Output() = 0;

    void addDeadline(float after);

    long int getTimeBeforeNextDeadline();

    void lazyEnd();

    wr_glyph *getBuffer();

    void resizeIfNeeded();

public:
    webrogue_event getEvent();

protected: // methods to override
    virtual void onBegin() = 0;

    virtual void onEnd() = 0;

    virtual void onBeginFrame() = 0;

    virtual void onEndFrame() = 0;

    virtual void onLazyEnd();
};

class EmptyOutput : public Output {
public:
    EmptyOutput();
    ~EmptyOutput() override;

protected:
    bool isKeyboardAvailable() override;

    void onBegin() override;

    void onEnd() override;

    void onBeginFrame() override;

    void onEndFrame() override;
};

} // namespace core
} // namespace webrogue
