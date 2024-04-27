#include "CompactLinking.hpp"
#include "../linker/compact-linker.h"
#include <cstddef>
#include <cstdint>
#include <functional>
#include <ios>
#include <memory>
#include <vector>

#if defined(NDEBUG) && defined(WEBROGUE_WASM_LD_PATH)
#error WEBROGUE_WASM_LD_PATH macro defined in release build
#endif
#if defined(WEBROGUE_WASM_LD_PATH) && !defined(WEBROGUE_WASM_LD_PLAYGROUND_PATH)
#error WEBROGUE_WASM_LD_PLAYGROUND_PATH macro must be defined when using WEBROGUE_WASM_LD_PATH
#endif

#ifdef WEBROGUE_WASM_LD_PATH
#include "../common/stringize.hpp"
#include <cassert>
#include <cstdlib>
#include <fstream>
#include <string>

#endif

namespace webrogue {
namespace core {
std::shared_ptr<std::vector<uint8_t>>
getCompactlyLinkedBinaries(ModsRuntime *runtime,
                           ResourceStorage *resourceStorage, Config *config,
                           std::function<void()> interrupt,
                           ConsoleStream *wrout, ConsoleStream *wrerr) {

#ifdef WEBROGUE_WASM_LD_PATH
    std::string const wasmLdPath = stringize(WEBROGUE_WASM_LD_PATH);
    std::string const wasmLdPlaygroundPath =
        stringize(WEBROGUE_WASM_LD_PLAYGROUND_PATH);

    assert(!wasmLdPath.empty());
    assert(!wasmLdPlaygroundPath.empty());

    std::string command;
    command = "mkdir -p " + wasmLdPlaygroundPath;
    system(command.c_str());
    command = "rm -f " + wasmLdPlaygroundPath + "*";
    system(command.c_str());

    std::string inputFilesArgs = "";
    auto writeFile = [&resourceStorage, &wasmLdPlaygroundPath](
                         std::string oldFileName, std::string newFileName) {
        assert(resourceStorage->hasFile(oldFileName));
        std::ofstream fout;
        fout.open(wasmLdPlaygroundPath + newFileName,
                  std::ios::binary | std::ios::out);
        assert(fout.is_open());
        auto file = resourceStorage->getFile(oldFileName);
        fout.write((char *)file.data(), file.size());

        fout.close();
    };

    for (std::string const modname : resourceStorage->modNames) {
        writeFile(modname + "/mod.a", modname + ".a");
        inputFilesArgs += " " + wasmLdPlaygroundPath + modname + ".a";
        inputFilesArgs += " --export=init_mod_" + modname;
    }
    writeFile("core/stdlibs.a", "stdlibs.a");

    std::string const outFilePath = wasmLdPlaygroundPath + "linked.wasm";

    command = wasmLdPath + " --export=wr_start" +
              " --export=__wasm_call_ctors" + " -zstack-size=5242880" +
              " --trace" + " --no-entry" + " --fatal-warnings" +
              " --no-gc-sections" + " --stack-first" +
              " --no-merge-data-segments" + inputFilesArgs + " -o " +
              outFilePath + " " + wasmLdPlaygroundPath + "stdlibs.a";

    system(command.c_str());

    std::ifstream fin;
    fin.open(outFilePath, std::ios::binary | std::ios::in | std::ios::ate);
    assert(fin.is_open());
    size_t const len = fin.tellg();
    fin.seekg(0, std::ios::beg);
    auto result = std::make_shared<std::vector<uint8_t>>();
    result->resize(len);
    fin.read((char *)result->data(), len);
    fin.close();
    return result;
#endif

    // {
    //     std::ifstream file("external/wabt/src/linking/"
    //                        "test_src/t1_lld_strip.wasm",
    //                        std::ios::in | std::ios::binary);
    //     assert(file.is_open());
    //     file.unsetf(std::ios::skipws);
    //     file.seekg(0, std::ios_base::end);
    //     size_t fileSize = file.tellg();
    //     file.seekg(0, std::ios_base::beg);
    //     std::vector<uint8_t> out;
    //     out.resize(0);
    //     out.reserve(fileSize);
    //     out.insert(out.begin(), std::istream_iterator<uint8_t>(file),
    //                std::istream_iterator<uint8_t>());
    //     return std::make_shared<std::vector<uint8_t>>(out);
    // }

    *wrout << "linking...\n";

    std::vector<LinkableFile> binaries;
    std::vector<std::string> requiredFuncs;

    for (std::string const modname : resourceStorage->modNames) {
        std::string const filename = modname + "/mod.a";
        if (!resourceStorage->hasFile(filename)) {
            *wrerr << "Could not find " + filename + " for linking\n";
            return nullptr;
        }
        binaries.push_back(
            LinkableFile(filename, resourceStorage->getFile(filename)));
        requiredFuncs.push_back("init_mod_" + modname);
    }
    requiredFuncs.push_back("wr_start");
    {
        std::string const filename = "core/stdlibs.a";

        if (!resourceStorage->hasFile(filename)) {
            *wrerr << "Could not find " + filename + " for linking\n";
            return nullptr;
        }
        binaries.push_back(
            LinkableFile(filename, resourceStorage->getFile(filename)));
    }

    return std::make_shared<std::vector<uint8_t>>(
        compact_link(requiredFuncs, binaries, interrupt));
}
} // namespace core
} // namespace webrogue
