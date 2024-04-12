#include "SDLDisplay.hpp"
#include "../../../external/libtsm/external/xkbcommon/xkbcommon-keysyms.h"
#include "../../core/EventManager.hpp"
#include "SDLTerminal.hpp"
#include "SDL_keycode.h"
#include <cstdint>
#include <cstring>
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
        switch (event.type) {
        case SDL_QUIT: {
            outputEvent.type = webrogue_event_type::Close;
            eventManager.addEvent(outputEvent);
        } break;
        case SDL_WINDOWEVENT: {
            switch (event.window.event) {
            case SDL_WINDOWEVENT_RESIZED:
                outputEvent.type = webrogue_event_type::Resize;
                eventManager.addEvent(outputEvent);
                break;
            case SDL_WINDOWEVENT_CLOSE:
                outputEvent.type = webrogue_event_type::Close;
                eventManager.addEvent(outputEvent);
                break;
            default:
                break;
            }
        } break;
        case SDL_KEYDOWN: {
            auto *sdlTerminal = reinterpret_cast<SDLTerminal *>(terminal.get());
            uint32_t sym = event.key.keysym.sym;
            unsigned int mods = 0;
            if (event.key.keysym.mod & KMOD_CTRL)
                mods |= TSM_CONTROL_MASK;
            if (sym == SDLK_BACKSPACE) {
                // mods |= TSM_CONTROL_MASK;
                sym = XKB_KEY_BackSpace;
            }
            if (sym == SDLK_RETURN) {
                // mods |= TSM_CONTROL_MASK;
                sym = XKB_KEY_Return;
                sdlTerminal->writeStdin("\n", 1);
            }

            sdlTerminal->ignoreStdin = true;
            sdlTerminal->keyPressed(sym, 0, mods, 0);
            sdlTerminal->ignoreStdin = false;
        } break;

        case SDL_TEXTINPUT: {
            auto *sdlTerminal = reinterpret_cast<SDLTerminal *>(terminal.get());
            sdlTerminal->feedText(event.text.text, strlen(event.text.text));
        }

        break;
        default:
            break;
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
    }
    auto *sdlTerminal = reinterpret_cast<SDLTerminal *>(terminal.get());
    sdlTerminal->poll(eventManager);
    sdlTerminal->bufferedDraw();
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
