#include "../../../src/common/load_embedded_mods.hpp"
#include "../../../src/core/webrogueMain.hpp"
#include "../../../src/outputs/sdl/SDLOutput.hpp"
#include <emscripten.h>
#include <memory>

void draw() {
}

int main(int argc, char *argv[]) {
    webrogue::core::Config config;
    load_embedded_mods(&config);
    emscripten_set_main_loop(draw, 0, 0);
    config.loadsModsFromDataPath = false;
    config.setDataPath("/webrogue");

    EM_ASM(FS.syncfs(true, function(err){}););
    config.onFrameEnd = []() {
        EM_ASM(FS.syncfs(function(err){}););
    };
    return webrogue::core::webrogueMain(
        std::make_shared<webrogue::outputs::sdl::SDLOutput>(),
        webrogue::runtimes::makeDefaultRuntime, &config);
}
