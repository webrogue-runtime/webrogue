#pragma once

#include "Output.hpp"
#include <memory>
#include <string>

namespace webrogue {
namespace core {
class ConsoleWriter {
    struct TextFragment {
        std::u32string s;
        int fd;
    };
    // -1 means scroll to end
    // -2 means scroll almost to end
    int scrollPos = -1;
    std::shared_ptr<Output> output;
    std::list<TextFragment> textFragments;

public:
    ConsoleWriter(std::shared_ptr<Output> output);
    bool isShown = true;
    void write(std::u32string data, bool isError);
    char igetc();
    void iungetc();
    void render(bool isQuick);
    void scrollUp();
    void scrollDown();
    webrogue_event present();
};
} // namespace core
} // namespace webrogue
