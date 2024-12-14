#import "../../external/SDL2/src/video/uikit/SDL_uikitappdelegate.h"
#import "main.h"

#define EGL_EGL_PROTOTYPES 1
#include <EGL/egl.h>
#include <EGL/eglext.h>

#define GL_GLES_PROTOTYPES 1
#include <GLES2/gl2.h>

#include "SDL.h"
#include "SDL_video.h"
#include "SDL_metal.h"

static const int WEBROGUE_SDL_WINDOWPOS_UNDEFINED = SDL_WINDOWPOS_UNDEFINED;
CAMetalLayer* wr_SDL_Metal_GetLayer(SDL_MetalView view);
