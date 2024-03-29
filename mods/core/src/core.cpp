#include "../include/core.h"
#include "../include/common/colors.h"
#include "../include/common/events.h"
#include "../include/macros.h"
#include "../include/wr_api.h"
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fcntl.h>
#include <fstream>
#include <queue>
#include <sstream>
#include <stddef.h>
#include <string>
#include <vector>

std::vector<webrogue_initialization_step> initializationSteps;
extern "C" void
webrogue_core_add_initialization_step(webrogue_initialization_step step) {
    initializationSteps.push_back(step);
}

std::vector<webrogue_event> eventBuffer;
extern "C" webrogue_event *webrogue_core_get_events(size_t *eventCount) {
    *eventCount = eventBuffer.size();
    return eventBuffer.data();
}

std::vector<wr_glyph> drawingArea;
size_t drawingAreaWidth;
size_t drawingAreaHeight;

extern "C" wr_glyph *webrogue_core_get_drawing_area(size_t *outWidth,
                                                    size_t *outHeight) {
    if (outWidth)
        *outWidth = drawingAreaWidth;
    if (outHeight)
        *outHeight = drawingAreaHeight;
    return drawingArea.data();
}

extern "C" void webrogue_core_print(const char *str) {
    wr_debug_print((uint64_t)str, strlen(str));
}

extern "C" WR_EXPORTED(void, wr_start)() {
    drawingAreaWidth = wr_get_render_width();
    drawingAreaHeight = wr_get_render_height();
    drawingArea.resize(drawingAreaWidth * drawingAreaHeight);
    for (webrogue_initialization_step step : initializationSteps) {
        step();
    }
    eventBuffer.clear();
    wr_render_set_screen_data((int64_t)drawingArea.data(), drawingArea.size());
}

extern "C" void webrogue_core_interrupt() {
    eventBuffer.clear();
    wr_render_set_screen_data((int64_t)drawingArea.data(), drawingArea.size());
    int32_t const numberOfEvents = wr_interrupt();
    eventBuffer.resize(numberOfEvents);
    if (numberOfEvents)
        wr_copy_events((int64_t)eventBuffer.data(), numberOfEvents);
    drawingAreaWidth = wr_get_render_width();
    drawingAreaHeight = wr_get_render_height();
    drawingArea.resize(drawingAreaWidth * drawingAreaHeight);
}

extern "C" WR_EXPORTED(void, init_mod_core)() {
    FILE *f = fopen("./text.txt", "w+");
    fwrite("test\n", 1, 5, f);
    fflush(f);
    fwrite("test2\n", 1, 6, f);
    fseek(f, 2, SEEK_SET);
    fwrite("00", 1, 2, f);
    fseek(f, -1, SEEK_CUR);
    fwrite("1", 1, 1, f);
    fseek(f, 0, SEEK_END);
    fwrite("test3", 1, 5, f);

    std::vector<char> filedata;
    fseek(f, 0, SEEK_END);
    filedata.resize(ftello(f));
    fseek(f, 0, SEEK_SET);
    fread(filedata.data(), filedata.size(), 1, f);

    fclose(f);

    f = fopen("./text2.txt", "w");
    fwrite(filedata.data(), filedata.size(), 1, f);
    fclose(f);
}

#if !defined(WEBROGUE_NATIVE_MODS)
int main() {
}
#endif
