#pragma once

#include <ostream>

namespace webrogue {
namespace core {
class ConsoleStream : public std::ostream {
    class TermBuf : public std::streambuf {
        bool isError;

    public:
        TermBuf(bool isError);

        virtual std::streamsize xsputn(const char *s,
                                       std::streamsize n) override;
    };

    TermBuf buf;
    std::streambuf *oldBuf = nullptr;

public:
    ConsoleStream(bool isError);
    ~ConsoleStream();
};
} // namespace core
} // namespace webrogue
