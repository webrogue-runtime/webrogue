#include "../../src/common/load_embedded_mods.hpp"
#include "../../src/core/webrogueMain.hpp"
#include "../../src/outputs/curses/CursesOutput.hpp"

int main(int argc, char *argv[]) {
    webrogue::core::Config config;
    load_embedded_mods(&config);
    return webrogue::core::webrogueMain(
        std::make_shared<webrogue::outputs::curses::CursesOutput>(),
        webrogue::runtimes::makeDefaultRuntime, &config);
}
