#include "SDL.h"
#include "SDL_video.h"
#include "webrogue_gfx_ffi.h"
#include <stdlib.h>

typedef struct System {

} System;

void *webrogue_gfx_ffi_create_system(void) {
  System *system_ptr = malloc(sizeof(System));
  SDL_Init(SDL_INIT_VIDEO);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_ES);
  SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);

  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 2);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 0);

  return system_ptr;
}
void webrogue_gfx_ffi_destroy_system(void *raw_system_ptr) {
  System *system_ptr = (System *)raw_system_ptr;
  free(system_ptr);
}
typedef struct Window {
  SDL_Window *sdl_window;
} Window;
void *webrogue_gfx_ffi_create_window(void *raw_system_ptr) {
  // System *system_ptr = (System *)raw_system_ptr;
  Window *window_ptr = malloc(sizeof(Window));

  window_ptr->sdl_window = SDL_CreateWindow(
    "webrogue", SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, 800, 450,
    SDL_WINDOW_OPENGL | SDL_WINDOW_RESIZABLE | SDL_WINDOW_ALLOW_HIGHDPI
  );
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
  SDL_GL_GetDrawableSize(window_ptr->sdl_window, &width, &height);
  *out_width = width;
  *out_height = height;
}
void webrogue_gfx_ffi_present_window(void *raw_window_ptr) {
  Window *window = (Window *)raw_window_ptr;
  SDL_GL_SwapWindow(window->sdl_window);
  SDL_Event event;
  SDL_PollEvent(&event);
}


static void *get_proc_address(char *procname, void *raw_system_ptr) {
  return SDL_GL_GetProcAddress(procname);
}
void* webrogue_gfx_ffi_gl_init(void *raw_window_ptr) {
  Window *window_ptr = (Window *)raw_window_ptr;
  SDL_GL_CreateContext(window_ptr->sdl_window);
  return (void*)get_proc_address;
}
