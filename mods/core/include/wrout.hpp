#pragma once

#include <ostream>

namespace core {
class wrostream : public std::ostream {
    class TermBuf : public std::streambuf {
        bool iserr;

    public:
        TermBuf(bool iserr);

        virtual std::streamsize xsputn(const char *s,
                                       std::streamsize n) override;
    };

    TermBuf buf;
    std::streambuf *oldBuf = nullptr;

public:
    wrostream(bool iserr);
    ~wrostream();
};

extern wrostream wrout;
extern wrostream wrerr;
} // namespace core