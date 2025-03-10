#pragma once
#include "webrogue_event_encoder.h"

void webrogue_gfx_ffi_sdl2_poll(webrogue_event_out_buf *event_buf,
                                void **out_buf, uint32_t *out_len);
