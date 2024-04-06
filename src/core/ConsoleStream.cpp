#include "ConsoleStream.hpp"
#include "utf.hpp"

namespace webrogue {
namespace core {
ConsoleStream::TermBuf::TermBuf(bool isError) : isError(isError) {
}

std::streamsize ConsoleStream::TermBuf::xsputn(const char *s,
                                               std::streamsize n) {
    // consoleWriter->write(utf::toUTF32(std::string((char *)s, n)), isError);
    return n;
}

ConsoleStream::ConsoleStream(bool isError)
    : buf(isError),
#ifdef __WINDOWS__
      std::ostream(&buf, false)
#else
      std::ostream(&buf)
#endif
{
    oldBuf = rdbuf(&buf);
}
ConsoleStream::~ConsoleStream() {
    rdbuf(oldBuf);
}
} // namespace core
} // namespace webrogue
