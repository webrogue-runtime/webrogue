#include "utf.hpp"

#include <cstddef>
#include <cstdint>
#include <string>

namespace webrogue {
namespace utf {
static const uint32_t offsetsFromUTF8[6] = {0x00000000UL, 0x00003080UL,
                                            0x000E2080UL, 0x03C82080UL,
                                            0xFA082080UL, 0x82082080UL};

static const uint8_t trailingBytesForUTF8[256] = {
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5};

bool bbxIsUTF8z(const uint8_t *str) {
    int len = 0;
    int pos = 0;
    int nb;
    int i;
    int ch;

    while (str[len])
        len++;
    while (pos < len && *str) {
        nb = bbxUTF8Skip(str);
        if (nb < 1 || nb > 4)
            return false;
        if (pos + nb > len)
            return false;
        for (i = 1; i < nb; i++)
            if ((str[i] & 0xC0) != 0x80)
                return false;
        ch = bbxUTF8Getch(str);
        if (ch < 0x80) {
            if (nb != 1)
                return false;
        } else if (ch < 0x8000) {
            if (nb != 2)
                return false;
        } else if (ch < 0x10000) {
            if (nb != 3)
                return false;
        } else if (ch < 0x110000) {
            if (nb != 4)
                return false;
        }
        pos += nb;
        str += nb;
    }

    return true;
}

size_t bbxUTF8Skip(const uint8_t *utf8) {
    return trailingBytesForUTF8[(uint8_t)*utf8] + 1;
}

uint32_t bbxUTF8Getch(const uint8_t *utf8) {
    int ch;
    int nb;

    nb = trailingBytesForUTF8[(uint8_t)*utf8];
    ch = 0;
    switch (nb) {
        /* these fall through deliberately */
    case 3:
        ch += (uint8_t)*utf8++;
        ch <<= 6;
    case 2:
        ch += (uint8_t)*utf8++;
        ch <<= 6;
    case 1:
        ch += (uint8_t)*utf8++;
        ch <<= 6;
    case 0:
        ch += (uint8_t)*utf8++;
    }
    ch -= offsetsFromUTF8[nb];

    return ch;
}

size_t bbxUTF8Putch(uint8_t *out, int ch) {
    uint8_t *dest = out;
    if (ch < 0x80) {
        *dest++ = (uint8_t)ch;
    } else if (ch < 0x800) {
        *dest++ = (ch >> 6) | 0xC0;
        *dest++ = (ch & 0x3F) | 0x80;
    } else if (ch < 0x10000) {
        *dest++ = (ch >> 12) | 0xE0;
        *dest++ = ((ch >> 6) & 0x3F) | 0x80;
        *dest++ = (ch & 0x3F) | 0x80;
    } else if (ch < 0x110000) {
        *dest++ = (ch >> 18) | 0xF0;
        *dest++ = ((ch >> 12) & 0x3F) | 0x80;
        *dest++ = ((ch >> 6) & 0x3F) | 0x80;
        *dest++ = (ch & 0x3F) | 0x80;
    } else
        return 0;
    return dest - out;
}

size_t bbxUTF8Charwidth(int ch) {
    if (ch < 0x80) {
        return 1;
    }
    if (ch < 0x800) {
        return 2;
    }
    if (ch < 0x10000) {
        return 3;
    }
    if (ch < 0x110000) {
        return 4;
    }
    return 0;
}

size_t bbxUTF8Nchars(const uint8_t *utf8) {
    int answer = 0;

    while (*utf8) {
        utf8 += bbxUTF8Skip(utf8);
        answer++;
    }

    return answer;
}

std::u32string toUTF32(std::string uft8string) {
    static_assert(sizeof(uint8_t) == sizeof(char), "what?");
    std::u32string result =
        std::u32string(bbxUTF8Nchars((uint8_t *)uft8string.c_str()), L' ');
    const uint8_t *utf8 = (uint8_t *)uft8string.c_str();
    int pos = 0;
    while (*utf8) {
        result.at(pos) = bbxUTF8Getch(utf8);
        utf8 += bbxUTF8Skip(utf8);
        pos++;
    }
    return result;
}
} // namespace utf
} // namespace webrogue