#pragma once

#include <cstddef>
#include <cstdint>
#include <string>

namespace webrogue {
namespace utf {
bool bbxIsUTF8z(const uint8_t *str);
size_t bbxUTF8Skip(const uint8_t *utf8);
uint32_t bbxUTF8Getch(const uint8_t *utf8);
size_t bbxUTF8Putch(uint8_t *out, int ch);
size_t bbxUTF8Charwidth(int ch);
size_t bbxUTF8Nchars(const uint8_t *utf8);
std::u32string toUTF32(std::string uft8string);
} // namespace utf
} // namespace webrogue
