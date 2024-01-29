#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

typedef uint32_t wr_color_pair;

typedef uint32_t wr_char;

typedef struct wr_glyph {
    wr_char unicode_char;
    wr_color_pair color;
} wr_glyph;
#ifdef __cplusplus
static_assert(sizeof(wr_glyph) == 8, "wrong wr_glyph size");
#endif

#ifdef __cplusplus
}
#endif
