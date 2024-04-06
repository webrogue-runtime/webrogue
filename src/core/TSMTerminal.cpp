#include "TSMTerminal.hpp"
#include "../../external/libtsm/src/tsm/libtsm.h"
#include <cstdint>
#include <cstdio>

namespace webrogue {
namespace core {

void tsmTerminalLog(void *data, const char *file, int line, const char *func,
                    const char *subs, unsigned int sev, const char *format,
                    va_list args) {
}
void tsmTerminalVTEWrite(struct tsm_vte *vte, const char *u8, size_t len,
                         void *data) {
    // stdin
}

int tsmTerminalDraw(struct tsm_screen *con, uint64_t id, const uint32_t *ch,
                    size_t len, unsigned int width, unsigned int posx,
                    unsigned int posy, const struct tsm_screen_attr *attr,
                    tsm_age_t age, void *data) {
    auto *tsmTerminal = reinterpret_cast<TSMTerminal *>(data);
    // len == 0 means just background
    tsmTerminal->drawGlyph(posx, posy, ch[0]);
    return 0;
}

TSMTerminal::TSMTerminal() {
    tsm_screen_new(&screen, tsmTerminalLog, this);
    tsm_vte_new(&vte, screen, tsmTerminalVTEWrite, this, tsmTerminalLog, this);
    tsm_vte_input(vte, "\e[20h", 5); // newline mode
}
void TSMTerminal::writeStdout(void const *data, size_t size) {
    tsm_screen_resize(screen, getWidth(), getHeight());
    tsm_vte_input(vte, (const char *)data, size);
    tsm_screen_draw(screen, tsmTerminalDraw, this);
}
TSMTerminal::~TSMTerminal() {
    tsm_vte_unref(vte);
    tsm_screen_unref(screen);
}
} // namespace core
} // namespace webrogue
