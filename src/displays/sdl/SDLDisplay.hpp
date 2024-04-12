#pragma once

#include "../../core/Display.hpp"
#include <SDL.h>
#include <SDL_ttf.h>
#include <optional>

namespace webrogue {
namespace displays {
namespace sdl {
class SDLDisplay : public core::Display {
public:
    SDLDisplay();
    SDL_Renderer *renderer;
    void poll(core::EventManager &eventManager) override;
    SDL_Window *window;
    ~SDLDisplay() override;

private:
    std::optional<SDL_KeyboardEvent> lastKeyboardEvent;
};
} // namespace sdl
} // namespace displays
} // namespace webrogue
