#include "CompactLinking.hpp"
#include "../linker/compact-linker.h"
#include <cstdint>
#include <functional>
#include <memory>
#include <vector>

namespace webrogue {
namespace core {
std::shared_ptr<std::vector<uint8_t>> getCompactlyLinkedBinaries(
    ModsRuntime *runtime, ResourceStorage *resourceStorage,
    Config const *config, std::function<void()> interrupt) {

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

    // *wrout << "linking...\n";

    std::vector<LinkableFile> binaries;
    std::vector<std::string> requiredFuncs;

    for (std::string const modname : resourceStorage->modNames) {
        std::string const filename = modname + "/mod.a";
        if (!resourceStorage->hasFile(filename)) {
            // *wrerr << "Could not find " + filename + " for linking\n";
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
            // *wrerr << "Could not find " + filename + " for linking\n";
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
