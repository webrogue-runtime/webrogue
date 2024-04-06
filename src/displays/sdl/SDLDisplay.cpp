#include "SDLDisplay.hpp"
#include "SDLTerminal.hpp"
#include <memory>

namespace webrogue {
namespace displays {
namespace sdl {
SDLDisplay::SDLDisplay() {
    SDL_Init(SDL_INIT_EVERYTHING);
    const int windowFlags = SDL_WINDOW_SHOWN | SDL_WINDOW_RESIZABLE
#if defined(__ANDROID__)
                            | SDL_WINDOW_FULLSCREEN;
#elif defined(TARGET_OS_MAC)
#elif defined(TARGET_OS_IPHONE)
                            | SDL_WINDOW_FULLSCREEN | SDL_WINDOW_BORDERLESS;
#endif
    ;
    SDL_CreateWindowAndRenderer(1280, 720, windowFlags, &window, &renderer);
    TTF_Init();
    terminal = std::make_unique<SDLTerminal>(this);
}
SDLDisplay::~SDLDisplay() {
    TTF_Quit();

    SDL_DestroyWindow(window);
    SDL_DestroyRenderer(renderer);
    SDL_Quit();
}
} // namespace sdl
} // namespace displays
} // namespace webrogue
