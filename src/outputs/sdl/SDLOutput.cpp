#include "SDLOutput.hpp"

#include "../../../embedded_resources/sdl_font_ttf.h"
#include "../../../mods/core/include/common/events.h"
#include "../../core/Output.hpp"
#include "SDL.h"
#include "SDL_render.h"
#include "SDL_timer.h"
#include "SDL_ttf.h"
#include <set>
#include <utility>

#if defined(__EMSCRIPTEN__)
#include "emscripten.h"
#endif

namespace webrogue {
namespace outputs {
namespace sdl {
bool WRGlyphComparator::operator()(const wr_glyph &a, const wr_glyph &b) const {
    if (a.unicode_char != b.unicode_char)
        return a.unicode_char < b.unicode_char;
    if (a.color != b.color)
        return a.color < b.color;
    return false;
}

bool SDLOutput::isKeyboardAvailable() {
#if defined(__ANDROID__)
    return false;
#elif defined(TARGET_OS_IPHONE)
    return false;
#else
    return true;
#endif
}

SDLOutput::SDLOutput() {
}

void SDLOutput::pollEvents() {
    SDL_Event event;
    webrogue_event outputEvent;

    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_QUIT) {
            outputEvent.type = webrogue_event_type::Close;
            events.push(outputEvent);
            continue;
        }
        if (event.type == SDL_WINDOWEVENT) {
            if (event.window.event == SDL_WINDOWEVENT_RESIZED) {
                outputEvent.type = webrogue_event_type::Resize;
                events.push(outputEvent);
                windowSizeChanged = true;
            }
            if (event.window.event == SDL_WINDOWEVENT_CLOSE) {
                outputEvent.type = webrogue_event_type::Close;
                events.push(outputEvent);
                continue;
            }
        }
        if (event.type == SDL_MOUSEBUTTONDOWN ||
            event.type == SDL_MOUSEBUTTONUP) {
            int const tx = (event.button.x - dx) / fontWidth;
            int const ty = (event.button.y - dy) / fontHeight;
            if (event.type == SDL_MOUSEBUTTONDOWN)
                outputEvent.type = webrogue_event_type::MouseLeftButtonPressed;
            else
                outputEvent.type = webrogue_event_type::MouseLeftButtonReleased;
            outputEvent.data1 = tx;
            outputEvent.data2 = ty;
            events.push(outputEvent);
            continue;
        }
        if (event.type == SDL_FINGERDOWN || event.type == SDL_FINGERUP) {
            int windowWidth = 0;
            int windowHeight = 0;
            SDL_GetWindowSize(window, &windowWidth, &windowHeight);
            int const tx = (event.tfinger.x * windowWidth - dx) / fontWidth;
            int const ty = (event.tfinger.y * windowHeight - dy) / fontHeight;
            if (event.type == SDL_FINGERDOWN)
                outputEvent.type = webrogue_event_type::MouseLeftButtonPressed;
            else
                outputEvent.type = webrogue_event_type::MouseLeftButtonReleased;
            outputEvent.data1 = tx;
            outputEvent.data2 = ty;
            events.push(outputEvent);
            continue;
        }

        if (event.type == SDL_KEYDOWN) {
            if (event.key.keysym.sym == 96) {
                outputEvent.type = webrogue_event_type::Console;
                events.push(outputEvent);
            }
            pressed.insert(event.key.keysym.sym);
            continue;
        }
        if (event.type == SDL_KEYUP) {
            pressed.erase(event.key.keysym.sym);
            continue;
        }
    }

    if ((std::chrono::steady_clock::now() - lastPress).count() > 200e6) {
        for (auto i = pressed.cbegin(); i != pressed.cend(); i++) {
            switch (*i) {
            case SDLK_UP:
                outputEvent.type = webrogue_event_type::Arrow;
                outputEvent.data1 = webrogue_arrow_direction::up;
                events.push(outputEvent);
                break;
            case SDLK_DOWN:
                outputEvent.type = webrogue_event_type::Arrow;
                outputEvent.data1 = webrogue_arrow_direction::down;
                events.push(outputEvent);
                break;
            case SDLK_LEFT:
                outputEvent.type = webrogue_event_type::Arrow;
                outputEvent.data1 = webrogue_arrow_direction::left;
                events.push(outputEvent);
                break;
            case SDLK_RIGHT:
                outputEvent.type = webrogue_event_type::Arrow;
                outputEvent.data1 = webrogue_arrow_direction::right;
                events.push(outputEvent);
                break;
            }
            lastPress = std::chrono::steady_clock::now();
        }
    }
    if (hasDeadline && getTimeBeforeNextDeadline() < 0) {
        events.push({webrogue_event_type::Deadline});
        hasDeadline = false;
    }
}

void SDLOutput::onBegin() {
    SDL_Init(SDL_INIT_EVERYTHING);
    int windowFlags = SDL_WINDOW_SHOWN | SDL_WINDOW_RESIZABLE;
#if defined(__ANDROID__)
    windowFlags |= SDL_WINDOW_RESIZABLE;
    windowFlags |= SDL_WINDOW_FULLSCREEN;
#elif defined(TARGET_OS_MAC)
#elif defined(TARGET_OS_IPHONE)
    windowFlags |= SDL_WINDOW_FULLSCREEN | SDL_WINDOW_BORDERLESS;
#endif
    SDL_CreateWindowAndRenderer(1280, 720, windowFlags, &window, &renderer);
    TTF_Init();
    colormap.resize(256);
    colorPairs.resize(256);
    lastInputValid = false;
}

void SDLOutput::onEnd() {
    if (font)
        TTF_CloseFont(font);
    TTF_Quit();

    SDL_DestroyWindow(window);
    SDL_DestroyRenderer(renderer);
    SDL_Quit();
}

void SDLOutput::onBeginFrame() {
    pollEvents();
    refreshSize();
    presentTexture();
}

void SDLOutput::refreshSize() {
    int windowWidth = 0;
    int windowHeight = 0;
    SDL_GetWindowSize(window, &windowWidth, &windowHeight);
    windowHeight -= topInset;
    int preferedFont = (windowWidth / maxWidth) * 2;
    preferedFont = std::max(preferedFont, windowHeight / maxHeight);
    preferedFont = std::max(preferedFont, 12);

    if (windowWidth != previousWindowWidth ||
        windowHeight != previousWindowHeight) {
        lastInputValid = false;
        if (font)
            TTF_CloseFont(font);
        font = TTF_OpenFontRW(
            SDL_RWFromMem((void *)sdl_font_ttf, sdl_font_ttf_size), 0,
            preferedFont);
        TTF_SizeText(font, "x", &fontWidth, &fontHeight);
        storedSize.x = std::min(maxWidth, windowWidth / fontWidth);
        storedSize.y = std::min(maxHeight, windowHeight / fontHeight);
        resizeIfNeeded();
        dx = (windowWidth - (storedSize.x * fontWidth)) / 2;
        dy = (windowHeight - (storedSize.y * fontHeight)) / 2;
        dy += topInset;
        previousWindowWidth = windowWidth;
        previousWindowHeight = windowHeight;
        for (auto i = glyph2tex.begin(); i != glyph2tex.end(); i++)
            SDL_DestroyTexture(i->second);
        glyph2tex.clear();

        lastInput.resize(storedSize);
    }
}

void SDLOutput::startColor() {
    setColor(0b000, 0, 0, 0);
    setColor(0b001, 1000, 0, 0);
    setColor(0b010, 0, 1000, 0);
    setColor(0b011, 1000, 1000, 0);
    setColor(0b100, 0, 0, 1000);
    setColor(0b101, 1000, 0, 1000);
    setColor(0b110, 0, 1000, 1000);
    setColor(0b111, 1000, 1000, 1000);

    setColorPair(0, 7, 0);
    setColorPair(1, 7, 1);
    setColorPair(2, 7, 2);
    setColorPair(3, 7, 3);
    setColorPair(4, 7, 4);
    setColorPair(5, 7, 5);
    setColorPair(6, 7, 6);
    setColorPair(7, 0, 7);
}

int32_t SDLOutput::getColorPairsCount() {
    return colorPairs.size();
}

int32_t SDLOutput::getColorsCount() {
    return colormap.size();
}

void SDLOutput::setColor(int32_t color, int32_t r, int32_t g, int32_t b) {
    if (color <= 0 || color >= colormap.size())
        return;
    colormap[color] = {(uint8_t)(r * 255 / 1000), (uint8_t)(g * 255 / 1000),
                       (uint8_t)(b * 255 / 1000), 0};
    for (auto i = glyph2tex.begin(); i != glyph2tex.end(); i++)
        SDL_DestroyTexture(i->second);
    glyph2tex.clear();
}

void SDLOutput::setColorPair(int32_t colorPair, int32_t fg, int32_t bg) {
    if (colorPair <= 0 || colorPair >= colorPairs.size())
        return;
    if (fg < 0 || fg >= colorPairs.size())
        return;
    if (bg < 0 || bg >= colorPairs.size())
        return;
    colorPairs[colorPair] = {fg, bg};
    for (auto i = glyph2tex.begin(); i != glyph2tex.end(); i++)
        SDL_DestroyTexture(i->second);
    glyph2tex.clear();
}

void SDLOutput::drawInputToTexture() {
    if (!lastInputValid) {
        SDL_DestroyTexture(lastTexture);
        lastTexture = nullptr;
    }
    if (!lastTexture) {
        lastTexture = SDL_CreateTexture(
            renderer, SDL_PIXELFORMAT_RGBA8888, SDL_TEXTUREACCESS_TARGET,
            storedSize.x * fontWidth, storedSize.y * fontHeight);

        SDL_SetRenderTarget(renderer, lastTexture);
        SDL_SetRenderDrawColor(renderer, 255, 0, 0, 0);
        SDL_RenderClear(renderer);
    }
    SDL_Rect rect;
    SDL_Surface *surface;
    for (int y = 0; y < storedSize.y; y++) {
        for (int x = 0; x < storedSize.x; x++) {
            wr_glyph const ch = renderBuffer.at(x, y);
            wr_glyph const lastCh = lastInput.at(x, y);
            if (!lastInputValid || WRGlyphComparator()(ch, lastCh) ||
                WRGlyphComparator()(lastCh, ch) || true) {
                lastInput.at(x, y) = ch;
                SDL_Texture *texture;
                if (glyph2tex.count(ch))
                    texture = glyph2tex[ch];
                else {
                    auto colorPair = colorPairs[ch.color];
                    SDL_Color backgroundColor = colormap[colorPair.bg];
                    SDL_Color foregroundColor = colormap[colorPair.fg];
                    if (!ch.color) {
                        foregroundColor = {255, 255, 255, 0};
                        backgroundColor = {0, 0, 0, 0};
                    }
                    wr_char const chars[2] = {0, 0};
                    surface = TTF_RenderGlyph32_Shaded(font, ch.unicode_char,
                                                       foregroundColor,
                                                       backgroundColor);
                    SDL_Texture *tmpTexture =
                        SDL_CreateTextureFromSurface(renderer, surface);
                    SDL_FreeSurface(surface);
                    texture = SDL_CreateTexture(
                        renderer, SDL_PIXELFORMAT_RGBA8888,
                        SDL_TEXTUREACCESS_TARGET, fontWidth, fontHeight);
                    SDL_SetRenderTarget(renderer, texture);
                    SDL_SetRenderDrawColor(renderer, backgroundColor.r,
                                           backgroundColor.g, backgroundColor.b,
                                           backgroundColor.a);
                    SDL_RenderClear(renderer);
                    SDL_RenderCopy(renderer, tmpTexture, NULL, NULL);
                    SDL_DestroyTexture(tmpTexture);

                    glyph2tex[ch] = texture;
                }
                SDL_SetRenderTarget(renderer, lastTexture);
                rect.x = x * fontWidth;
                rect.y = y * fontHeight;
                rect.w = fontWidth;
                rect.h = fontHeight;
                SDL_RenderCopy(renderer, texture, NULL, &rect);
            }
        }
    }
    fpsSync();
}
void SDLOutput::fpsSync() {
    uint32_t const passed = SDL_GetTicks() - lastUpdateTick;
    constexpr uint32_t minFrameTime = 1000 / 100;
    if (passed < minFrameTime) {
#if defined(__EMSCRIPTEN__)
        emscripten_sleep(minFrameTime - passed);
#else
        SDL_Delay(minFrameTime - passed);
#endif
    }
    lastUpdateTick = SDL_GetTicks();
}

void SDLOutput::presentTexture() {
    SDL_SetRenderTarget(renderer, NULL);
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 0);
    SDL_RenderClear(renderer);
    SDL_Rect rect;
    rect.x = dx;
    rect.y = dy;
    rect.w = fontWidth * lastInput.size().x;
    rect.h = fontHeight * lastInput.size().y;
    SDL_RenderCopy(renderer, lastTexture, NULL, &rect);
    SDL_RenderPresent(renderer);
}

void SDLOutput::onLazyEnd() {
    presentTexture();
    fpsSync();
}

void SDLOutput::onEndFrame() {
    drawInputToTexture();
    presentTexture();

    lastInputValid = true;
}

SDLOutput::~SDLOutput() {
}
} // namespace sdl
} // namespace outputs
} // namespace webrogue
