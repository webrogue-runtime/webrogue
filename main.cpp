
#include "src/core/ModsRuntime.hpp"
#include <memory>
#if defined(OUTPUT_SDL)
#include "src/outputs/sdl/SDLOutput.hpp"
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
    webrogue::core::Config config;
#if !defined(WEBROGUE_NATIVE_MODS)
    config.addWrmodData(core_wrmod, core_wrmod_size, "core");
    config.loadsModsFromDataPath = true;
#else
    load_embedded_mods(&config);
    config.loadsModsFromDataPath = false;
#endif
    config.setDataPath(".");

#if defined(OUTPUT_SDL)
    return webrogueMain(std::make_shared<webrogue::outputs::sdl::SDLOutput>(),
                        webrogue::runtimes::makeDefaultRuntime, &config);
#else
    return webrogueMain(
        std::make_shared<webrogue::outputs::curses::CursesOutput>(),
        webrogue::runtimes::makeDefaultRuntime, &config);
#endif
}
