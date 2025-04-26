#import "../../external/SDL3/src/video/uikit/SDL_uikitappdelegate.h"
#import "main.h"

#define EGL_EGL_PROTOTYPES 1
#include <EGL/egl.h>
#include <EGL/eglext.h>

#define GL_GLES_PROTOTYPES 1
#include <GLES2/gl2.h>

#include "../../external/SDL3/include/SDL3/SDL.h"
#include "../../external/SDL3/include/SDL3/SDL_video.h"
#include "../../external/SDL3/include/SDL3/SDL_metal.h"

const int WR_SDL_WINDOW_METAL = SDL_WINDOW_METAL;
const int WR_SDL_WINDOW_RESIZABLE = SDL_WINDOW_RESIZABLE;
const int WR_SDL_WINDOW_HIGH_PIXEL_DENSITY = SDL_WINDOW_HIGH_PIXEL_DENSITY;

static const int WEBROGUE_SDL_WINDOWPOS_UNDEFINED = SDL_WINDOWPOS_UNDEFINED;
CAMetalLayer* wr_SDL_Metal_GetLayer(SDL_MetalView view);
