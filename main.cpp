
#include "core/Terminal.hpp"
#include "src/core/ModsRuntime.hpp"
#include <memory>
#if defined(WEBROGUE_DISPLAY_SDL)
#include "src/displays/sdl/SDLDisplay.hpp"
#else
#include "src/outputs/curses/CursesOutput.hpp"
#endif
#include "src/core/webrogueMain.hpp"

#if !defined(WEBROGUE_NATIVE_MODS)
#include "embedded_resources/core_wrmod.h"
#else
#include "src/common/load_embedded_mods.hpp"
#endif

#if defined(_WIN32)
int WinMain() {
#else
int main(int argc, char *argv[]) {
#endif
    webrogue::core::Config config(".");
#if !defined(WEBROGUE_NATIVE_MODS)
    config.setModsData(core_wrmod, core_wrmod_size, "core", false);
#else
    load_embedded_mods(&config);
    config.loadsModsFromDataPath = false;
#endif

#if defined(WEBROGUE_DISPLAY_SDL)
    config.setDisplay(std::make_shared<webrogue::displays::sdl::SDLDisplay>());
#else
    !!!terminal =
        std::make_shared<webrogue::terminals::emulated::EmulatedTerminal>();
#endif
    return webrogueMain(webrogue::runtimes::makeDefaultRuntime, config);
}
