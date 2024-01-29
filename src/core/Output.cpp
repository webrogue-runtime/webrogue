#include "Output.hpp"
#include <algorithm>
#include <cstring>

namespace webrogue {
namespace core {
Vec2Int Output::size() {
    return storedSize;
}

void Output::begin() {
    if (!isRendering) {
        onBegin();
        isRendering = true;
    }
}

void Output::end() {
    if (isRendering) {
        endFrame();
        isRendering = false;
    }
    onEnd();
}

void Output::beginFrame() {
    begin();
    if (!isRenderingFrame) {
        onBeginFrame();
        storedSize.x = std::min(storedSize.x, maxWidth);
        storedSize.y = std::min(storedSize.y, maxHeight);
        isRenderingFrame = true;
    }
}

void Output::endFrame() {
    if (renderBuffer.size() != storedSize) {
        lazyEnd();
        isRenderingFrame = false;
        return;
    }
    if (isRenderingFrame) {
        onEndFrame();
        isRenderingFrame = false;
    }
}

webrogue_event Output::getEvent() {
    if (events.empty())
        return {webrogue_event_type::None};
    webrogue_event result = events.top();
    events.pop();
    return result;
}

void Output::addDeadline(float after) {
    std::chrono::microseconds ms{static_cast<long int>(after * 1000000)};
    auto newDeadline = std::chrono::steady_clock::now() + ms;
    deadline = std::min(deadline, newDeadline);
    hasDeadline = true;
}

long int Output::getTimeBeforeNextDeadline() {
    if (!hasDeadline)
        return -1;

    return std::chrono::duration_cast<std::chrono::milliseconds>(
               deadline - std::chrono::steady_clock::now())
        .count();
}

void Output::lazyEnd() {
    onLazyEnd();
    isRenderingFrame = false;
}

wr_glyph *Output::getBuffer() {
    return &renderBuffer.at(0, 0);
}

void Output::resizeIfNeeded() {
    if (renderBuffer.size() != storedSize) {
        renderBuffer.resize(storedSize);
        wr_glyph c = {U' ', 0};
        renderBuffer.fill(c);
    }
}

void Output::onLazyEnd() {
}

Output::~Output() {
}

EmptyOutput::EmptyOutput(){};
EmptyOutput::~EmptyOutput(){};

bool EmptyOutput::isKeyboardAvailable() {
    return true;
};

void EmptyOutput::onBegin(){};

void EmptyOutput::onEnd(){};

void EmptyOutput::onBeginFrame(){};

void EmptyOutput::onEndFrame(){};

} // namespace core
} // namespace webrogue
