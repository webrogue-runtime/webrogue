#include "../../src/core/webrogueMain.hpp"
#include "../../src/outputs/sdl/SDLOutput.hpp"
#include "common.hpp"

int WinMain() {
    webrogue::core::Config config;
    initConfig(config);
    return webrogue::core::webrogueMain(
        std::make_shared<webrogue::outputs::sdl::SDLOutput>(),
        webrogue::runtimes::makeDefaultRuntime, &config);
}
