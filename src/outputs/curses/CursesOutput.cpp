#include "CursesOutput.hpp"

#include "../../../mods/core/include/common/events.h"
#include "../../core/utf.hpp"
#include "curses_include.hpp"
#include <clocale>

namespace webrogue {
namespace outputs {
namespace curses {

CursesOutput::CursesOutput() {
}

bool CursesOutput::isKeyboardAvailable() {
    return true;
}

void CursesOutput::pollEvent(int milliseconds) {
    timeout(milliseconds);
    int const r = getch();
    webrogue_event event;
    if (r == ERR) {
        event.type = webrogue_event_type::None;
        events.push(event);
        return;
    }
    if (r == 3) {
        event.type = webrogue_event_type::Close;
        events.push(event);
        return;
    }
    if (r == KEY_MOUSE) {
        MEVENT mouseEvent;
        if (nc_getmouse(&mouseEvent) == OK) {
            event.data1 = mouseEvent.x - dx;
            event.data2 = mouseEvent.y - dy;
            event.type = webrogue_event_type::None;
            if (mouseEvent.bstate & BUTTON1_PRESSED) {
                event.type = webrogue_event_type::MouseLeftButtonPressed;
            } else if (mouseEvent.bstate & BUTTON1_RELEASED) {
                event.type = webrogue_event_type::MouseLeftButtonReleased;
            } else {
                event.type = webrogue_event_type::MouseMoved;
            }
            if (event.type != webrogue_event_type::None) {
                events.push(event);
                return;
            };
        }
    }
    { // Arrows
        event.type = webrogue_event_type::Arrow;
        switch (r) {
        case 261: // right
            event.data1 = webrogue_arrow_direction::right;
            events.push(event);
            return;
        case 260: // left
            event.data1 = webrogue_arrow_direction::left;
            events.push(event);
            return;
        case 259: // up
            event.data1 = webrogue_arrow_direction::up;
            events.push(event);
            return;
        case 258: // down
            event.data1 = webrogue_arrow_direction::down;
            events.push(event);
            return;
        default:
            break;
        }
    }
    { // Chars
        if (r >= 'A' && r <= 'z') {
            event.type = webrogue_event_type::Char;
            event.data1 = r;
            events.push(event);
            return;
        }
    }
    if (r == KEY_RESIZE) {
        event.type = webrogue_event_type::Resize;
        events.push(event);
        return;
    }
    if (r == '`') {
        event.type = webrogue_event_type::Console;
        events.push(event);
        return;
    }
    if (r == -1) {
        doupdate();
    }
    return;
}

void CursesOutput::onBegin() {
    def_shell_mode();
    setlocale(LC_ALL, "");
    initscr();
    keypad(stdscr, true);

    mousemask(BUTTON1_PRESSED | BUTTON1_RELEASED | REPORT_MOUSE_POSITION,
              nullptr);
    mouseinterval(0);

    raw();
    noecho();
    scrollok(stdscr, true);
    def_prog_mode();
}

void CursesOutput::startColor() {
    ::start_color();
    clear();
    doupdate();
}

int32_t CursesOutput::getColorPairsCount() {
    return COLORS;
};

int32_t CursesOutput::getColorsCount() {
    return COLOR_PAIRS;
};

void CursesOutput::setColor(int32_t color, int32_t r, int32_t g, int32_t b) {
    init_color(color, r, g, b);
}

void CursesOutput::setColorPair(int32_t colorPair, int32_t fg, int32_t bg) {
    init_pair(colorPair, fg, bg);
}

void CursesOutput::onEnd() {
    endwin();
}

void CursesOutput::onBeginFrame() {
    while (events.empty()) {
        int time = -1;
        if (hasDeadline) {
            time = getTimeBeforeNextDeadline();
            time = (time > 0 ? time : 0);
        }
        pollEvent(time);
        if (hasDeadline && getTimeBeforeNextDeadline() < 0) {
            events.push({webrogue_event_type::Deadline});
            hasDeadline = false;
        }
    }
    getmaxyx(stdscr, storedSize.y, storedSize.x);
    storedSize.y = std::min(storedSize.y, maxHeight);
    storedSize.x = std::min(storedSize.x, maxWidth);
    resizeIfNeeded();
}

void CursesOutput::onEndFrame() {
    Vec2Int currentSize(0, 0);
    static Vec2Int lastSize(0, 0);
    getmaxyx(stdscr, currentSize.y, currentSize.x);
    if (lastSize != currentSize) {
        clear();
        lastSize = currentSize;
    }
    dx = (currentSize.x - storedSize.x) / 2;
    dy = (currentSize.y - storedSize.y) / 2;
    for (int y = 0; y < storedSize.y; y++) {
        move(y + dy, dx);
        for (int x = 0; x < storedSize.x; x++) {
            if (x == storedSize.x - 1 && y == storedSize.y - 1)
                continue;
            wr_glyph const ch = renderBuffer.at(x, y);
            attron(COLOR_PAIR(ch.color));
            union {
                uint64_t whole;
                uint8_t chars[8];
            } outChar;
            outChar.whole = 0;
            utf::bbxUTF8Putch(&(outChar.chars[0]),
                              ch.unicode_char ? ch.unicode_char : U' ');

            addstr((char *)&(outChar.chars[0]));
            attroff(COLOR_PAIR(ch.color));
        }
    }
    refresh();
}
CursesOutput::~CursesOutput() {
}
} // namespace curses
} // namespace outputs
} // namespace webrogue
