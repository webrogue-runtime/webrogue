#pragma once
#include <stdint.h>

void *webrogue_gfx_ffi_create_system(void);
void webrogue_gfx_ffi_destroy_system(void *raw_system_ptr);
void *webrogue_gfx_ffi_create_window(void *raw_system_ptr);
void webrogue_gfx_ffi_destroy_window(void *raw_window_ptr);
void webrogue_gfx_ffi_get_window_size(void *raw_window_ptr, uint32_t *out_width,
                                      uint32_t *out_height);
void webrogue_gfx_ffi_get_gl_size(void *raw_window_ptr, uint32_t *out_width,
                                  uint32_t *out_height);
void webrogue_gfx_ffi_present_window(void *raw_window_ptr);
void webrogue_gfx_ffi_gl_init(void *raw_window_ptr, void** out_func, void** out_userdata);
void webrogue_gfx_ffi_poll(void *raw_system_ptr, void** out_buf, uint32_t* out_len);
void webrogue_gfx_ffi_get_gl_swap_interval(void *raw_system_ptr, uint32_t* out_interval);
