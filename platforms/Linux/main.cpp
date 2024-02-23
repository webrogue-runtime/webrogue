#include "../../src/core/webrogueMain.hpp"
#include "../../src/outputs/curses/CursesOutput.hpp"
#include "../embedded_resources/core_wrmod.h"
#include "find_data_path.hpp"

__asm__(".symver realpath,realpath@GLIBC_2.2.5");
int main(int argc, char *argv[]) {
    webrogue::config::Config config;
    config.addWrmodData(core_wrmod, core_wrmod_size, "core");
    config.setModsPath(findModsPath());
    config.setDataPath(findDataPath());
    config.loadsModsFromDataPath = true;
    return webrogue::core::webrogueMain(
        std::make_shared<webrogue::output::CursesOutput>(),
        webrogue::runtimes::makeDefaultRuntime, &config);
}
