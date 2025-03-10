#pragma once
// GENERATED BY webrogue-event-apigen. DO NOT EDIT MANUALLY
#include <stdint.h>

typedef struct webrogue_event_out_buf {
    void* buf;
    uint32_t buf_size;
    uint32_t used_size;
} webrogue_event_out_buf;

#define BUF_SIZE(LEN) if(out->buf_size < LEN) { out->used_size = 0; return; } out->used_size = LEN
#define SET(TYPE, OFFSET, VALUE) *((TYPE*)(((char*)out->buf) + OFFSET)) = VALUE

static inline void webrogue_event_encode_mouse_down(webrogue_event_out_buf *out, uint32_t x, uint32_t y, uint32_t button) {
    BUF_SIZE(16);
    SET(uint32_t, 0, 1);
    SET(uint32_t, 4, x);
    SET(uint32_t, 8, y);
    SET(uint32_t, 12, button);
}
static inline void webrogue_event_encode_mouse_up(webrogue_event_out_buf *out, uint32_t x, uint32_t y, uint32_t button) {
    BUF_SIZE(16);
    SET(uint32_t, 0, 2);
    SET(uint32_t, 4, x);
    SET(uint32_t, 8, y);
    SET(uint32_t, 12, button);
}
static inline void webrogue_event_encode_mouse_motion(webrogue_event_out_buf *out, uint32_t x, uint32_t y) {
    BUF_SIZE(12);
    SET(uint32_t, 0, 3);
    SET(uint32_t, 4, x);
    SET(uint32_t, 8, y);
}

#undef BUF_SIZE
#undef SET
