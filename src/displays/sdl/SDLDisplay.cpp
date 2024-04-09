#include "SDLDisplay.hpp"
#include "../../core/EventManager.hpp"
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

void SDLDisplay::poll(core::EventManager &eventManager) {
    SDL_Event event;
    webrogue_event outputEvent;

    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_QUIT) {
            outputEvent.type = webrogue_event_type::Close;
            eventManager.addEvent(outputEvent);
            continue;
        }
        if (event.type == SDL_WINDOWEVENT) {
            if (event.window.event == SDL_WINDOWEVENT_RESIZED) {
                outputEvent.type = webrogue_event_type::Resize;
                eventManager.addEvent(outputEvent);
            }
            if (event.window.event == SDL_WINDOWEVENT_CLOSE) {
                outputEvent.type = webrogue_event_type::Close;
                eventManager.addEvent(outputEvent);
                continue;
            }
        }
        // if (event.type == SDL_MOUSEBUTTONDOWN ||
        //     event.type == SDL_MOUSEBUTTONUP) {
        //     int const tx = (event.button.x - dx) / fontWidth;
        //     int const ty = (event.button.y - dy) / fontHeight;
        //     if (event.type == SDL_MOUSEBUTTONDOWN)
        //         outputEvent.type =
        //         webrogue_event_type::MouseLeftButtonPressed;
        //     else
        //         outputEvent.type =
        //         webrogue_event_type::MouseLeftButtonReleased;
        //     outputEvent.data1 = tx;
        //     outputEvent.data2 = ty;
        //     events.push(outputEvent);
        //     continue;
        // }
        // if (event.type == SDL_FINGERDOWN || event.type == SDL_FINGERUP) {
        //     int windowWidth = 0;
        //     int windowHeight = 0;
        //     SDL_GetWindowSize(window, &windowWidth, &windowHeight);
        //     int const tx = (event.tfinger.x * windowWidth - dx) / fontWidth;
        //     int const ty = (event.tfinger.y * windowHeight - dy) /
        //     fontHeight; if (event.type == SDL_FINGERDOWN)
        //         outputEvent.type =
        //         webrogue_event_type::MouseLeftButtonPressed;
        //     else
        //         outputEvent.type =
        //         webrogue_event_type::MouseLeftButtonReleased;
        //     outputEvent.data1 = tx;
        //     outputEvent.data2 = ty;
        //     events.push(outputEvent);
        //     continue;
        // }

        // if (event.type == SDL_KEYDOWN) {
        //     if (event.key.keysym.scancode == 96) {
        //         outputEvent.type = webrogue_event_type::Console;
        //         events.push(outputEvent);
        //     } else {
        //         switch (event.key.keysym.sym) {
        //         case SDLK_UP:
        //             outputEvent.type = webrogue_event_type::Arrow;
        //             outputEvent.data1 = webrogue_arrow_direction::up;
        //             events.push(outputEvent);
        //             break;
        //         case SDLK_DOWN:
        //             outputEvent.type = webrogue_event_type::Arrow;
        //             outputEvent.data1 = webrogue_arrow_direction::down;
        //             events.push(outputEvent);
        //             break;
        //         case SDLK_LEFT:
        //             outputEvent.type = webrogue_event_type::Arrow;
        //             outputEvent.data1 = webrogue_arrow_direction::left;
        //             events.push(outputEvent);
        //             break;
        //         case SDLK_RIGHT:
        //             outputEvent.type = webrogue_event_type::Arrow;
        //             outputEvent.data1 = webrogue_arrow_direction::right;
        //             events.push(outputEvent);
        //             break;
        //         case SDLK_KP_ENTER:
        //             outputEvent.type = webrogue_event_type::Key;
        //             outputEvent.data1 = 0x157;
        //             events.push(outputEvent);
        //             break;
        //         case SDLK_BACKSPACE:
        //             outputEvent.type = webrogue_event_type::Key;
        //             outputEvent.data1 = '\177';
        //             events.push(outputEvent);
        //             break;
        //         default:
        //             outputEvent.type = webrogue_event_type::Key;
        //             outputEvent.data1 = event.key.keysym.sym;
        //             events.push(outputEvent);
        //             break;
        //         }
        //     }
        //     continue;
        // }
        // if (event.type == SDL_TEXTINPUT) {
        //     for (int i = 0; i < 32 && event.text.text[i]; i++) {
        //         outputEvent.type = webrogue_event_type::Char;
        //         outputEvent.data1 = event.text.text[i];
        //         events.push(outputEvent);
        //     }
        //     continue;
        // }
    }
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
