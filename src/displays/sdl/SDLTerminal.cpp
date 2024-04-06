#include "SDLTerminal.hpp"
#include "../../../embedded_resources/hack_ttf.h"
#include "SDLDisplay.hpp"
#include "SDL_render.h"
#include <memory>

namespace webrogue {
namespace displays {
namespace sdl {
SDLTerminal::SDLTerminal(SDLDisplay *display) : display(display) {
    int windowWidth = 0;
    int windowHeight = 0;
    SDL_GetWindowSize(display->window, &windowWidth, &windowHeight);
    mainTexture =
        SDL_CreateTexture(display->renderer, SDL_PIXELFORMAT_RGBA8888,
                          SDL_TEXTUREACCESS_TARGET, windowWidth, windowHeight);
}

void SDLTerminal::writeStdout(void const *data, size_t size) {
    int windowWidth = 0;
    int windowHeight = 0;
    SDL_GetWindowSize(display->window, &windowWidth, &windowHeight);

    fontHeight = 12;

    if (font)
        TTF_CloseFont(font);
    font = TTF_OpenFontRW(SDL_RWFromMem((void *)hack_ttf, hack_ttf_size), 0,
                          fontHeight);
    TTF_SizeText(font, "x", &fontWidth, &fontHeight);

    charCountX = windowWidth / fontWidth;
    charCountY = windowHeight / fontHeight;

    dx = (windowWidth - (charCountX * fontWidth)) / 2;
    dy = (windowHeight - (charCountY * fontHeight)) / 2;

    SDL_SetRenderTarget(display->renderer, mainTexture);
    SDL_SetRenderDrawColor(display->renderer, 0, 0, 0, 0);
    SDL_RenderClear(display->renderer);

    core::TSMTerminal::writeStdout(data, size);
    // drawGlyph(3, 3, 'a');

    SDL_SetRenderTarget(display->renderer, NULL);
    SDL_RenderCopy(display->renderer, mainTexture, NULL, NULL);
    SDL_RenderPresent(display->renderer);
};

bool operator<(const SDLTerminal::GlyphMapKey &k1,
               const SDLTerminal::GlyphMapKey &k2) {
    return k1.ch < k2.ch;
}
SDLTerminal::GlyphMapValue::GlyphMapValue(SDL_Texture *texture)
    : texture(texture) {
}
SDLTerminal::GlyphMapValue::~GlyphMapValue() {
    SDL_DestroyTexture(texture);
}

void SDLTerminal::drawGlyph(int x, int y, uint32_t glyph) {
    SDL_Color foregroundColor = {255, 255, 255, 0};
    SDL_Color backgroundColor = {0, 0, 0, 0};
    GlyphMapKey key = {glyph};
    if (!glyphMap.count(key)) {
        SDL_Surface *surface = TTF_RenderGlyph32_Shaded(
            font, glyph, foregroundColor, backgroundColor);
        SDL_Texture *glyphTexture =
            SDL_CreateTextureFromSurface(display->renderer, surface);
        SDL_FreeSurface(surface);
        glyphMap[key] = std::make_unique<GlyphMapValue>(glyphTexture);
    }
    auto &value = glyphMap[key];

    SDL_Rect rect;
    SDL_SetRenderTarget(display->renderer, mainTexture);
    rect.x = x * fontWidth + dx;
    rect.y = y * fontHeight + dy;
    rect.w = fontWidth;
    rect.h = fontHeight;
    SDL_RenderCopy(display->renderer, value->texture, NULL, &rect);
}

int SDLTerminal::getWidth() {
    return charCountX;
}
int SDLTerminal::getHeight() {
    return charCountY;
}
SDLTerminal::~SDLTerminal() {
    SDL_DestroyTexture(mainTexture);
}
} // namespace sdl
} // namespace displays

} // namespace webrogue