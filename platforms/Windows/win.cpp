#include "../../src/core/webrogueMain.hpp"
#include "../../src/outputs/curses/CursesOutput.hpp"
#include "common.hpp"

int main() {
    webrogue::core::Config config;
    initConfig(config);
    return webrogue::core::webrogueMain(
        std::make_shared<webrogue::outputs::curses::CursesOutput>(),
        webrogue::runtimes::makeDefaultRuntime, &config);
}
