#include "../include/wrout.hpp"
#include "../include/core.h"
#include <string>

namespace core {
wrostream::TermBuf::TermBuf(bool pIserr) : iserr(pIserr) {
}

wrostream::wrostream(bool pIserr)
    : buf(pIserr),
#ifdef __WINDOWS__
      std::ostream(&buf, false)
#else
      std::ostream(&buf)
#endif
{
    oldBuf = rdbuf(&buf);
}
wrostream::~wrostream() {
    rdbuf(oldBuf);
}

std::streamsize wrostream::TermBuf::xsputn(const char *s, std::streamsize n) {
    // termFDs[fd]->ifwrite(s, 1, n);
    std::string const str(s, n);
    webrogue_core_print(str.c_str());

    return n;
}

wrostream wrout{false};
wrostream wrerr{true};

} // namespace core