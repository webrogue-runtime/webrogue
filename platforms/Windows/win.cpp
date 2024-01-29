#include "../../src/core/webrogueMain.hpp"
#include "../../src/outputs/curses/CursesOutput.hpp"

int main() {
    webrogue::core::Config config;
    config.setDataPath(".");
    config.loadsModsFromDataPath = true;
    return webrogue::core::webrogueMain(
        std::make_shared<webrogue::outputs::curses::CursesOutput>(),
        webrogue::runtimes::makeDefaultRuntime, &config);
}
