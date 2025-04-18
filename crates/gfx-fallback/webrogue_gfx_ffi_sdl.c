#include "webrogue_gfx_ffi.h"
#include "webrogue_gfx_ffi_sdl_events.h"
#include <stdlib.h>
#include <stdio.h>
#if WEBROGUE_GFX_SDL_VERSION == 2
#include "SDL.h"
#include "SDL_video.h"
#elif WEBROGUE_GFX_SDL_VERSION == 3
#include <SDL3/SDL.h>
#include <SDL3/SDL_video.h>
#else
#error Unknown WEBROGUE_GFX_SDL_VERSION value
#endif

typedef struct System {
  webrogue_event_out_buf event_buf;
} System;

void *webrogue_gfx_ffi_create_system(void) {
  System *system_ptr = malloc(sizeof(System));
  system_ptr->event_buf = webrogue_event_out_buf_create();
  SDL_Init(SDL_INIT_VIDEO);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_ES);
  SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);

  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 0);

  return system_ptr;
}
void webrogue_gfx_ffi_destroy_system(void *raw_system_ptr) {
  System *system_ptr = (System *)raw_system_ptr;
  webrogue_event_out_buf_delete(system_ptr->event_buf);
  free(system_ptr);
}
typedef struct Window {
  SDL_Window *sdl_window;
} Window;
void *webrogue_gfx_ffi_create_window(void *raw_system_ptr) {
  (void)raw_system_ptr;
  // System *system_ptr = (System *)raw_system_ptr;
  Window *window_ptr = malloc(sizeof(Window));
#if WEBROGUE_GFX_SDL_VERSION == 2
  window_ptr->sdl_window = SDL_CreateWindow(
      "webrogue", SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, 800, 450,
      SDL_WINDOW_OPENGL | SDL_WINDOW_RESIZABLE | SDL_WINDOW_ALLOW_HIGHDPI);
#elif WEBROGUE_GFX_SDL_VERSION == 3
  window_ptr->sdl_window = SDL_CreateWindow(
      "webrogue", 800, 450,
      SDL_WINDOW_OPENGL | SDL_WINDOW_RESIZABLE | SDL_WINDOW_HIGH_PIXEL_DENSITY);

#else
#error Unknown WEBROGUE_GFX_SDL_VERSION value
#endif
  return window_ptr;
}
void webrogue_gfx_ffi_destroy_window(void *raw_window_ptr) {
  Window *window_ptr = (Window *)raw_window_ptr;
  SDL_DestroyWindow(window_ptr->sdl_window);
  free(window_ptr);
}
void webrogue_gfx_ffi_get_window_size(void *raw_window_ptr, uint32_t *out_width,
                                      uint32_t *out_height) {
  Window *window_ptr = (Window *)raw_window_ptr;
  int width, height;
  SDL_GetWindowSize(window_ptr->sdl_window, &width, &height);
  *out_width = width;
  *out_height = height;
}
void webrogue_gfx_ffi_get_gl_size(void *raw_window_ptr, uint32_t *out_width,
                                  uint32_t *out_height) {
  Window *window_ptr = (Window *)raw_window_ptr;
  int width, height;
#if WEBROGUE_GFX_SDL_VERSION == 2
  SDL_GL_GetDrawableSize(window_ptr->sdl_window, &width, &height);
#elif WEBROGUE_GFX_SDL_VERSION == 3
  SDL_GetWindowSizeInPixels(window_ptr->sdl_window, &width, &height);

#else
#error Unknown WEBROGUE_GFX_SDL_VERSION value
#endif
  *out_width = width;
  *out_height = height;
}
void webrogue_gfx_ffi_present_window(void *raw_window_ptr) {
  Window *window = (Window *)raw_window_ptr;
  SDL_GL_SwapWindow(window->sdl_window);
  SDL_Event event;
  SDL_PollEvent(&event);
}

static void *get_proc_address(char *procname, void *userdata) {
  (void)userdata;
  void * result = SDL_GL_GetProcAddress(procname);
  // if(!result) {
  //   printf("SDL_GL_GetProcAddress(\"%s\") returned NULL\n", procname);
  // }
  return result;
}
void webrogue_gfx_ffi_gl_init(void *raw_window_ptr, void **out_func,
                              void **out_userdata) {
  Window *window_ptr = (Window *)raw_window_ptr;
  SDL_GL_CreateContext(window_ptr->sdl_window);
  *out_func = get_proc_address;
  *out_userdata = NULL;
}

void webrogue_gfx_ffi_poll(void *raw_system_ptr, void **out_buf,
                           uint32_t *out_len) {
  System *system_ptr = (System *)raw_system_ptr;
  webrogue_event_out_buf *event_buf = &(system_ptr->event_buf);
  webrogue_gfx_ffi_sdl_poll(event_buf, out_buf, out_len);
}
