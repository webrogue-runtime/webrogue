#pragma once

#include "../../core/Display.hpp"
#include <SDL.h>
#include <SDL_ttf.h>

namespace webrogue {
namespace displays {
namespace sdl {
class SDLDisplay : public core::Display {
public:
    SDLDisplay();
    SDL_Renderer *renderer;
    SDL_Window *window;
    ~SDLDisplay() override;
};
} // namespace sdl
} // namespace displays
} // namespace webrogue
