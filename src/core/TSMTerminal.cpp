#include "TSMTerminal.hpp"
#include "../../external/libtsm/src/tsm/libtsm-int.h"
#include "../../external/libtsm/src/tsm/libtsm.h"
#include "EventManager.hpp"
#include "utf.hpp"
#include <cstdint>
#include <cstdio>
#include <cstring>

namespace webrogue {
namespace core {

void tsmTerminalLog(void *data, const char *file, int line, const char *func,
                    const char *subs, unsigned int sev, const char *format,
                    va_list args) {
}
void tsmTerminalVTEWrite(struct tsm_vte *vte, const char *u8, size_t len,
                         void *data) {
    auto *tsmTerminal = reinterpret_cast<TSMTerminal *>(data);
    tsmTerminal->writeStdin(u8, len);
}

int tsmTerminalDraw(struct tsm_screen *con, uint64_t id, const uint32_t *ch,
                    size_t len, unsigned int width, unsigned int posx,
                    unsigned int posy, const struct tsm_screen_attr *attr,
                    tsm_age_t age, void *data) {
    auto *tsmTerminal = reinterpret_cast<TSMTerminal *>(data);
    // len == 0 means just background
    TSMTerminal::GlyphColor color;

    color.foregroundRed = attr->fr;
    color.foregroundGreen = attr->fg;
    color.foregroundBlue = attr->fb;
    color.backgroundRed = attr->br;
    color.backgroundGreen = attr->bg;
    color.backgroundBlue = attr->bb;

    color.foregroundCode = attr->fccode;
    color.backgroundCode = attr->bccode;

    tsmTerminal->drawGlyph(posx, posy, ch[0], color);
    return 0;
}

TSMTerminal::TSMTerminal() {
    tsm_screen_new(&screen, tsmTerminalLog, this);
    tsm_vte_new(&vte, screen, tsmTerminalVTEWrite, this, tsmTerminalLog, this);
    tsm_vte_input(vte, "\e[20h\e[12l", 10); // newline mode
}
void TSMTerminal::feedText(void const *data, size_t size) {
    auto utf32str = utf::toUTF32(std::string((const char *)data, size));
    for (auto utf32 : utf32str) {
        tsm_vte_handle_keyboard(vte, 0, 0, 0, utf32);
    }

    bufferedDraw();
}
void TSMTerminal::writeStdout(void const *data, size_t size) {
    tsm_screen_resize(screen, getWidth(), getHeight());
    tsm_vte_input(vte, (const char *)data, size);

    bufferedDraw();
}
void TSMTerminal::bufferedDraw() {
    auto now = std::chrono::steady_clock::now();
    auto duration = now - lastDrawTimePoint;
    using fps_60 = std::chrono::duration<double, std::ratio<1, 60>>;
    if (fps_60(duration).count() > 1) {
        draw();
        lastDrawTimePoint = now;
    }
}
void TSMTerminal::draw() {
    tsm_screen_draw(screen, tsmTerminalDraw, this);
}
void TSMTerminal::writeStdin(void const *data, size_t size) {
    if (ignoreStdin)
        return;
    stdinBuffer.resize(stdinBuffer.size() + size);
    memcpy(stdinBuffer.data() + stdinBuffer.size() - size, data, size);
}
void TSMTerminal::poll(EventManager &eventManager) {
    if (!stdinBuffer.empty()) {
        eventManager.writeStdin(stdinBuffer);
        stdinBuffer.clear();
    }
};
TSMTerminal::~TSMTerminal() {
    tsm_vte_unref(vte);
    tsm_screen_unref(screen);
}
} // namespace core
} // namespace webrogue
