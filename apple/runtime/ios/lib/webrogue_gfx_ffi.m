#include "../../../../crates/gfx-fallback/webrogue_gfx_ffi.h"
#include "../../../../crates/gfx-fallback/webrogue_gfx_ffi_sdl2_events.h"
#include "SDL.h"
#include "SDL_metal.h"
#include "SDL_video.h"
#import <QuartzCore/CAMetalLayer.h>
#include <stdlib.h>
#import <wrios-Swift.h>

typedef struct System {
    AngleHelperSystem *angle_helper_system;
    webrogue_event_out_buf event_buf;
} System;
typedef struct Window {
    AngleHelperWindow *angle_helper_window;
} Window;

void *webrogue_gfx_ffi_create_system(void) {
    System *system_ptr = malloc(sizeof(System));
    system_ptr->angle_helper_system = [[AngleHelperSystem alloc] init];
    system_ptr->event_buf.buf = malloc(1024);
    system_ptr->event_buf.buf_size = 1024;
    system_ptr->event_buf.used_size = 0;

    return system_ptr;
}
void webrogue_gfx_ffi_destroy_system(void *raw_system_ptr) {
    System *system_ptr = (System *)raw_system_ptr;
    system_ptr->angle_helper_system = nil;
    free(raw_system_ptr);
}
void *webrogue_gfx_ffi_create_window(void *raw_system_ptr) {
    System *system_ptr = (System *)raw_system_ptr;
    Window *window_ptr = malloc(sizeof(Window));
    window_ptr->angle_helper_window =
    [system_ptr->angle_helper_system makeWindow];
    return window_ptr;
}
void webrogue_gfx_ffi_destroy_window(void *raw_window_ptr) {
    Window *window_ptr = (Window *)raw_window_ptr;

    window_ptr->angle_helper_window = nil;
    free(window_ptr);
}
void webrogue_gfx_ffi_get_window_size(void *raw_window_ptr, uint32_t *out_width,
                                      uint32_t *out_height) {
    Window *window_ptr = (Window *)raw_window_ptr;
    *out_width = (uint32_t)[window_ptr->angle_helper_window viewWidth];
    *out_height = (uint32_t)[window_ptr->angle_helper_window viewHeight];
}
void webrogue_gfx_ffi_get_gl_size(void *raw_window_ptr, uint32_t *out_width,
                                  uint32_t *out_height) {
    Window *window_ptr = (Window *)raw_window_ptr;
    *out_width = (uint32_t)[window_ptr->angle_helper_window viewportWidth];
    *out_height = (uint32_t)[window_ptr->angle_helper_window viewportHeight];
}
void webrogue_gfx_ffi_present_window(void *raw_window_ptr) {
    Window *window_ptr = (Window *)raw_window_ptr;
    [window_ptr->angle_helper_window present];
    SDL_Event event;
    SDL_PollEvent(&event);
}
static void *get_proc_address(char *procname, void *userdata) {
    (void)userdata;
    return SDL_GL_GetProcAddress(procname);
}
void webrogue_gfx_ffi_gl_init(void *raw_window_ptr, void** out_func, void** out_userdata) {
    Window *window_ptr = (Window *)raw_window_ptr;
    [window_ptr->angle_helper_window initGL];
    [window_ptr->angle_helper_window makeCurrent];
    *out_func = get_proc_address;
    *out_userdata = window_ptr;
}
CAMetalLayer *wr_SDL_Metal_GetLayer(SDL_MetalView view) {
    return CFBridgingRelease(SDL_Metal_GetLayer(view));
}
void webrogue_gfx_ffi_poll(void *raw_system_ptr, void** out_buf, uint32_t* out_len) {
  System *system_ptr = (System *)raw_system_ptr;
  webrogue_event_out_buf *event_buf = &(system_ptr->event_buf);
  webrogue_gfx_ffi_sdl2_poll(event_buf, out_buf, out_len);
}
