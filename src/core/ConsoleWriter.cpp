#include "ConsoleWriter.hpp"
#include <cstring>

namespace webrogue {
namespace core {
ConsoleWriter::ConsoleWriter(std::shared_ptr<Output> output) : output(output){};

// TODO steal utf8 from lua
size_t getCharSize(const std::string &str, int i) {
    if (str[i] == '\0')
        return 0;
    return 1;
}

char ConsoleWriter::igetc() {
    return getc(stdin);
}

void ConsoleWriter::iungetc() {
}

void ConsoleWriter::write(std::u32string data, bool isError) {
    textFragments.push_back({data, isError ? 2 : 1});
    if (isError)
        isShown = true;
    // if (isShown && data.find('\n') != std::string::npos)
    render(true);
}

void ConsoleWriter::scrollUp() {
    if (scrollPos < 0)
        scrollPos = -2;
    else
        scrollPos = std::max(0, scrollPos - 1);
}

void ConsoleWriter::scrollDown() {
    if (scrollPos < 0)
        scrollPos = -1;
    else
        scrollPos++;
}

void ConsoleWriter::render(bool isQuick) {
    if (isQuick)
        output->addDeadline(0);
    Buffer2d<wr_glyph> buffer;
    Vec2Int size = output->size();
    buffer.resize(size);
    wr_glyph defaultGlyph;
    defaultGlyph.unicode_char = U' ';
    defaultGlyph.color = 0;
    buffer.resize(size);
    buffer.fill(defaultGlyph);
    int yOffset = 0;
    for (int isDrawing = 0; isDrawing < 2; isDrawing++) {
        int x = 0;
        int y = yOffset;
        for (TextFragment textFragment : textFragments) {
            for (size_t charI = 0; charI < textFragment.s.size(); charI++) {
                char c = textFragment.s[charI];
                if (isDrawing && y >= 0 && y < size.y) {
                    buffer.at(x, y) = wr_glyph();

                    if (c != '\n' && c != '\t')
                        buffer.at(x, y).unicode_char = textFragment.s[charI];
                    buffer.at(x, y).color = 0;
                }
                if (c == '\n') {
                    y++;
                    x = 0;
                } else if (c == '\t') {
                    x = 8 * (x / 8 + 1);
                    if (x >= size.x) {
                        y++;
                        x = 0;
                    }
                } else {
                    x++;
                    if (x >= size.x) {
                        y++;
                        x = 0;
                    }
                }
            }
        }
        if (!isDrawing)
            if (y >= size.y - 1) {
                yOffset = size.y - y - 1;
                if (scrollPos == -2) {
                    scrollPos = -yOffset - 1;
                }
                if (scrollPos != -1) {
                    int newYOffset = -scrollPos;
                    if (newYOffset <= yOffset) {
                        scrollPos = -1;
                    } else {
                        yOffset = newYOffset;
                    }
                }
            }
    }
    memcpy(output->getBuffer(), buffer.data(),
           buffer.size().x * buffer.size().y * sizeof(wr_glyph));
    output->endFrame();
    output->beginFrame();
}

webrogue_event ConsoleWriter::present() {
    while (true) {
        while (true) {
            auto event = output->getEvent();
            switch (event.type) {
            case webrogue_event_type::Arrow:
                if (event.data1 == webrogue_arrow_direction::up)
                    scrollUp();
                if (event.data1 == webrogue_arrow_direction::down)
                    scrollDown();
                break;
            case webrogue_event_type::Console:
                isShown = false;
                return event;
            case webrogue_event_type::Close:
                isShown = false;
                return event;
            default:
                break;
            }
            if (event.type == webrogue_event_type::None)
                break;
        }
        render(false);
    }
}

} // namespace core
} // namespace webrogue
