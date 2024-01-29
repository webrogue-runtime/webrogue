#pragma once

#include "ConsoleWriter.hpp"
#include <ostream>

namespace webrogue {
namespace core {
class ConsoleStream : public std::ostream {
    class TermBuf : public std::streambuf {
        ConsoleWriter *consoleWriter;
        bool isError;

    public:
        TermBuf(ConsoleWriter *consoleWriter, bool isError);

        virtual std::streamsize xsputn(const char *s,
                                       std::streamsize n) override;
    };

    TermBuf buf;
    std::streambuf *oldBuf = nullptr;

public:
    ConsoleStream(ConsoleWriter *consoleWriter, bool isError);
    ~ConsoleStream();
};
} // namespace core
} // namespace webrogue
