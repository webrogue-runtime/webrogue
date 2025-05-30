#include "webrogue_gfx_ffi.h"
#include "webrogue_gfx_ffi_sdl_events.h"
#include <SDL3/SDL.h>
#include <SDL3/SDL_video.h>
#include <stdio.h>
#include <stdlib.h>

#ifdef WEBROGUE_GFX_IOS
#define EGL_EGL_PROTOTYPES 1
#include <EGL/egl.h>
#include <EGL/eglext.h>

#define GL_GLES_PROTOTYPES 1
#include <GLES2/gl2.h>

#include "SDL3/SDL_metal.h"
#include "SDL3/SDL_video.h"
#endif

typedef struct System {
  webrogue_event_out_buf event_buf;
} System;

void *webrogue_gfx_ffi_create_system(void) {
  System *system_ptr = malloc(sizeof(System));
  system_ptr->event_buf = webrogue_event_out_buf_create();
  if (!SDL_Init(SDL_INIT_VIDEO)) {
    printf("SDL_Init failed: %s\n", SDL_GetError());
  };
#ifndef WEBROGUE_GFX_IOS
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_ES);
  SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 0);
#ifndef __EMSCRIPTEN__
  SDL_SetHint(SDL_HINT_OPENGL_LIBRARY, getenv("SDL_VIDEO_GL_DRIVER"));
  SDL_SetHint(SDL_HINT_EGL_LIBRARY, getenv("SDL_VIDEO_EGL_DRIVER"));
  SDL_GL_LoadLibrary(getenv("SDL_VIDEO_GL_DRIVER"));
#endif
#endif

  return system_ptr;
}
void webrogue_gfx_ffi_destroy_system(void *raw_system_ptr) {
  System *system_ptr = (System *)raw_system_ptr;
  webrogue_event_out_buf_delete(system_ptr->event_buf);
  free(system_ptr);
}
typedef struct Window {
  SDL_Window *sdl_window;
#ifdef WEBROGUE_GFX_IOS
  SDL_MetalView *sdl_metal_view;
  void *metal_layer;
  EGLDisplay *display;
  EGLSurface *surface;
  EGLContext *context;
#endif
} Window;
void *webrogue_gfx_ffi_create_window(void *raw_system_ptr) {
  (void)raw_system_ptr;
  // System *system_ptr = (System *)raw_system_ptr;
  Window *window_ptr = malloc(sizeof(Window));
#ifdef WEBROGUE_GFX_IOS
  int graphics_api_flag = SDL_WINDOW_METAL;
#else
  int graphics_api_flag = SDL_WINDOW_OPENGL;
#endif
  window_ptr->sdl_window = SDL_CreateWindow(
      "webrogue", 800, 450,
      graphics_api_flag | SDL_WINDOW_RESIZABLE | SDL_WINDOW_HIGH_PIXEL_DENSITY);
#ifdef WEBROGUE_GFX_IOS
  window_ptr->sdl_metal_view = SDL_Metal_CreateView(window_ptr->sdl_window);
  window_ptr->metal_layer = SDL_Metal_GetLayer(window_ptr->sdl_metal_view);
  window_ptr->display =
      eglGetPlatformDisplay(EGL_PLATFORM_ANGLE_ANGLE, NULL, NULL);
  if (!window_ptr->display) {
    printf("eglGetPlatformDisplay() returned error %d\n", eglGetError());
    abort();
  }

  if (!eglInitialize(window_ptr->display, NULL, NULL)) {
    printf("eglInitialize() returned error %d\n", eglGetError());
    abort();
  }
  EGLint configAttribs[] = {
      EGL_BLUE_SIZE,  8,  EGL_GREEN_SIZE, 8, EGL_RED_SIZE, 8,
      EGL_DEPTH_SIZE, 24, EGL_NONE,
  };
  EGLConfig config;
  EGLint numConfigs;
  if (!eglChooseConfig(window_ptr->display, &configAttribs, &config, 1,
                       &numConfigs)) {
    printf("eglChooseConfig() returned error %d\n", eglGetError());
    abort();
  }
  if (!numConfigs) {
    printf("eglChooseConfig() returned zero configs\n");
    abort();
  }

  EGLint contextAttribs[] = {
      EGL_CONTEXT_MAJOR_VERSION, 3, EGL_CONTEXT_MINOR_VERSION, 0, EGL_NONE,
  };

  window_ptr->context =
      eglCreateContext(window_ptr->display, config, NULL, &contextAttribs);
  if (!window_ptr->context) {
    printf("eglCreateContext() returned error %d\n", eglGetError());
    abort();
  }

  window_ptr->surface = eglCreateWindowSurface(
      window_ptr->display, config, (EGLNativeWindowType)window_ptr->metal_layer,
      NULL);
  if (!window_ptr->surface) {
    printf("eglCreateWindowSurface() returned error %d\n", eglGetError());
    abort();
  }
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
  SDL_GetWindowSizeInPixels(window_ptr->sdl_window, &width, &height);
  *out_width = width;
  *out_height = height;
}
void webrogue_gfx_ffi_present_window(void *raw_window_ptr) {
  Window *window_ptr = (Window *)raw_window_ptr;
#ifdef WEBROGUE_GFX_IOS
  eglSwapBuffers(window_ptr->display, window_ptr->surface);
#else
  SDL_GL_SwapWindow(window_ptr->sdl_window);
#endif
  SDL_PumpEvents();
}

static void *get_proc_address(char *procname, void *userdata) {
  (void)userdata;
  void *result = SDL_GL_GetProcAddress(procname);
  // if(!result) {
  //   printf("SDL_GL_GetProcAddress(\"%s\") returned NULL\n", procname);
  // }
  return result;
}
void webrogue_gfx_ffi_gl_init(void *raw_window_ptr, void **out_func,
                              void **out_userdata) {
  Window *window_ptr = (Window *)raw_window_ptr;
  SDL_GLContext gl_context = SDL_GL_CreateContext(window_ptr->sdl_window);
  if (!gl_context) {
    printf("SDL_GL_CreateContext failed: %s\n", SDL_GetError());
  };
#ifdef WEBROGUE_GFX_IOS
  eglMakeCurrent(window_ptr->display, window_ptr->surface, window_ptr->surface,
                 window_ptr->context);
#endif
  *out_func = get_proc_address;
  *out_userdata = NULL;
}

void webrogue_gfx_ffi_poll(void *raw_system_ptr, void **out_buf,
                           uint32_t *out_len) {
  System *system_ptr = (System *)raw_system_ptr;
  webrogue_event_out_buf *event_buf = &(system_ptr->event_buf);
  webrogue_gfx_ffi_sdl_poll(event_buf, out_buf, out_len);
}

void webrogue_gfx_ffi_get_gl_swap_interval(void *raw_system_ptr,
                                           uint32_t *out_interval) {
  (void)raw_system_ptr;
  int interval;
  if (!SDL_GL_GetSwapInterval(&interval)) {
    fprintf(stderr, "SDL error: %s\n", SDL_GetError());
  }
  *out_interval = interval;
}
