#include "../../../../crates/gfx-fallback/webrogue_gfx_ffi.h"
#include "SDL.h"
#include "SDL_metal.h"
#include "SDL_video.h"
#import <QuartzCore/CAMetalLayer.h>
#include <stdlib.h>
#import <webrogue-Swift.h>

typedef struct System {
  AngleHelperSystem *angle_helper_system;
} System;

void *webrogue_gfx_ffi_create_system(void) {
  System *system_ptr = malloc(sizeof(System));
  system_ptr->angle_helper_system = [[AngleHelperSystem alloc] init];

  return system_ptr;
}
void webrogue_gfx_ffi_destroy_system(void *raw_system_ptr) {
  System *system_ptr = (System *)raw_system_ptr;
  system_ptr->angle_helper_system = nil;
  free(raw_system_ptr);
}
typedef struct Window {
  AngleHelperWindow *angle_helper_window;
} Window;
void *webrogue_gfx_ffi_create_window(void *raw_system_ptr) {
  System *system_ptr = (System *)raw_system_ptr;
  Window *window_ptr = malloc(sizeof(Window));
  window_ptr->angle_helper_window =
      [system_ptr->angle_helper_system makeWindow];
  [window_ptr->angle_helper_window makeCurrent];
  return window_ptr;
}
void webrogue_gfx_ffi_destroy_window(void *raw_window_ptr) {
  Window *window_ptr = (Window *)raw_window_ptr;

  window_ptr->angle_helper_window = nil;
  free(window_ptr);
}
void *webrogue_gfx_ffi_gl_get_proc_address(void *raw_system_ptr,
                                           char *procname) {
  return SDL_GL_GetProcAddress(procname);
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

CAMetalLayer *wr_SDL_Metal_GetLayer(SDL_MetalView view) {
  return CFBridgingRelease(SDL_Metal_GetLayer(view));
}
