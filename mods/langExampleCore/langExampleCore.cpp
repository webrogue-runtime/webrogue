#include "../core/include/core.h"
#include "../core/include/macros.h"
#include <cstdlib>
#include <map>
#include <string>
extern "C" {
#include "langExampleCore.h"
}
#include <stdio.h>

std::map<std::string, langExampleFunc> exampleFuncs;
std::string langExampleResult;

extern "C" void addLangExample(const char *name, langExampleFunc func) {
    exampleFuncs[name] = func;
}

extern "C" void langExampleReturn(const char *result) {
    langExampleResult = result;
}

void langExampleCoreInitializationStep() {
    bool hasLastResult = false;
    while (true) {
        size_t eventCount;
        webrogue_event *events = webrogue_core_get_events(&eventCount);
        bool hasMouseEvent = false;
        int mouseX, mouseY;
        for (int i = 0; i < eventCount; i++) {
            webrogue_event const event = events[i];
            switch (event.type) {
            case Close:
                exit(0);
            case MouseLeftButtonPressed:
                hasMouseEvent = true;
                mouseX = event.data1;
                mouseY = event.data2;
                break;
            default:
                break;
            }
        }

        size_t width, height;
        wr_glyph *drawingArea = webrogue_core_get_drawing_area(&width, &height);
        for (int x = 0; x < width; x++)
            for (int y = 0; y < height; y++)
                drawingArea[width * y + x] = {' ', 0};
        int y = 0;
        auto printStr = [width, height, drawingArea, &y](std::string str) {
            int x = 0;
            for (int i = 0; i < str.size(); i++) {
                if (str[i] == '\n') {
                    x = 0;
                    y++;
                } else {
                    drawingArea[width * y + x] = {static_cast<wr_char>(str[i]),
                                                  0};
                    x++;
                    if (x >= width) {
                        x = 0;
                        y++;
                    }
                }
            }
            y++;
        };
        for (auto &pair : exampleFuncs) {
            if (hasMouseEvent && mouseY == y) {
                hasLastResult = true;
                pair.second();
            }
            printStr("> " + pair.first);
        }
        if (hasLastResult) {
            printStr("");
            printStr(langExampleResult);
        }
        webrogue_core_interrupt();
    }
}

void langExampleCore() {
    langExampleReturn("langExampleCore");
}

extern "C" WR_EXPORTED(void, init_mod_langExampleCore)() {
    webrogue_core_add_initialization_step(langExampleCoreInitializationStep);

    addLangExample("initial item", langExampleCore);
}
