#pragma once

#include "../../core/TSMTerminal.hpp"
#include <SDL.h>
#include <SDL_ttf.h>
#include <cstdint>
#include <map>
#include <memory>

namespace webrogue {
namespace displays {
namespace sdl {
class SDLDisplay;
class SDLTerminal : public core::TSMTerminal {
public:
    SDLTerminal(SDLDisplay *display);
    void writeStdout(void const *data, size_t size) override;
    int getWidth() override;
    int getHeight() override;
    void refreshWindowSize();
    void draw() override;
    void keyPressed(uint32_t keysym, uint32_t ascii, unsigned int mods,
                    uint32_t unicode);

    ~SDLTerminal() override;

    class GlyphMapKey {
    public:
        uint32_t ch;
        friend bool operator<(const GlyphMapKey &k1, const GlyphMapKey &k2);
    };
    class GlyphMapValue {
    public:
        GlyphMapValue(SDL_Texture *texture);
        SDL_Texture *texture;
        ~GlyphMapValue();
    };

private:
    SDLDisplay *display;
    SDL_Texture *mainTexture = 0;
    TTF_Font *font = nullptr;

    std::map<GlyphMapKey, std::unique_ptr<GlyphMapValue>> glyphMap;

    int fontHeight, fontWidth;

    int windowWidth, windowHeight;

    int charCountX, charCountY;
    int dx, dy;

    void drawGlyph(int x, int y, uint32_t glyph, GlyphColor color) override;
};
} // namespace sdl
} // namespace displays
} // namespace webrogue
