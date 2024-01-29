#include "../../src/core/webrogueMain.hpp"
#include "../../src/outputs/sdl/SDLOutput.hpp"

int WinMain() {
    webrogue::core::Config config;
    config.setDataPath(".");
    config.loadsModsFromDataPath = true;
    return webrogue::core::webrogueMain(
        std::make_shared<webrogue::outputs::sdl::SDLOutput>(),
        webrogue::runtimes::makeDefaultRuntime, &config);
}
