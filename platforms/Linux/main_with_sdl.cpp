#include "../../embedded_resources/core_wrmod.h"
#include "../../external/argparse/include/argparse/argparse.hpp"
#include "../../src/core/webrogueMain.hpp"
#include "../../src/outputs/curses/CursesOutput.hpp"
#include "../../src/outputs/sdl/SDLOutput.hpp"
#include "find_data_path.hpp"
#include <cstdio>
#include <cstdlib>
#include <iostream>
#include <memory>
#include <string>

int main(int argc, char *argv[]) {
    argparse::ArgumentParser program("webrogue");
    program.add_argument("-o", "--output")
        .default_value(std::string("auto"))
        .help("output type to use, or \"list\" to get list of available output "
              "types")
        .required();
    try {
        program.parse_args(argc, argv);
    } catch (const std::runtime_error &err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        std::exit(1);
    }
    std::string outputType = program.get<std::string>("--output");

    std::shared_ptr<webrogue::core::Output> output = nullptr;
    if (outputType == "auto") {
        outputType =
            getenv("WAYLAND_DISPLAY") || getenv("DISPLAY") ? "sdl" : "curses";
    }
    if (outputType == "curses") {
        output = std::make_shared<webrogue::outputs::curses::CursesOutput>();
    } else if (outputType == "sdl") {
        output = std::make_shared<webrogue::outputs::sdl::SDLOutput>();
    } else if (outputType == "list") {
        std::cerr << "curses" << std::endl
                  << "sdl" << std::endl
                  << "auto" << std::endl;
        return 1;
    } else {
        std::cerr << "output type " << outputType << "not available"
                  << std::endl
                  << "try \"webrogue --output list\" to get list of available "
                     "output types"
                  << std::endl;
        return 1;
    }

    webrogue::core::Config config;
    config.addWrmodData(core_wrmod, core_wrmod_size, "core");

    config.setModsPath(findModsPath());
    config.setDataPath(findDataPath());
    config.loadsModsFromDataPath = true;

    return webrogue::core::webrogueMain(
        output, webrogue::runtimes::makeDefaultRuntime, &config);
}

//