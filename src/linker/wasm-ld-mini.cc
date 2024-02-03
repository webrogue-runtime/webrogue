#include "archive-reader.h"
#include "stream.h"
#include "synthesizer.h"
#include "write_file.h"

#include <cstdlib>
#include <string>

#include "../common/stringize.hpp"

using namespace wabt;
Result run() {
    // system("pwd");
    string const modNamesStr = std::string(stringize(WEBROGUE_MOD_NAMES)) + "|";
    string const delimiter = "|";

    ArchiveReader archiveReader;

    string copiedModNamesStr = modNamesStr;
    size_t pos = 0;
    while ((pos = copiedModNamesStr.find(delimiter)) != std::string::npos) {
        string const funcName = "init_mod_" + copiedModNamesStr.substr(0, pos);
        archiveReader.addRequiredFunction(funcName.c_str());
        copiedModNamesStr.erase(0, pos + delimiter.length());
    }
    archiveReader.addRequiredFunction("wr_start");

    system("rm -f actual");

    copiedModNamesStr = modNamesStr;
    pos = 0;
    while ((pos = copiedModNamesStr.find(delimiter)) != std::string::npos) {
        string const fileName =
            "../mods/" + copiedModNamesStr.substr(0, pos) + "/mod.a";
        CHECK_RESULT(archiveReader.addFile(fileName.c_str()));
        copiedModNamesStr.erase(0, pos + delimiter.length());
    }
    CHECK_RESULT(archiveReader.addFile("../mods/core/stdlibs.a"));

    archiveReader.synthesize();
    Synthesizer synthesizer;
    synthesizer.stackSize = 5242880;
    WASMModule outputModule;
    auto symbols = archiveReader.symbols;
    CHECK_RESULT(synthesizer.synthesize(&archiveReader.modules,
                                        &archiveReader.symbols, &outputModule));
    MemoryStream stream;
    // // FileStream stream{};
    CHECK_RESULT(write_file(&outputModule, &stream));
    stream.Flush();
    stream.WriteToFile("../src/linker/linked_my.wasm");

    string objdumpCommand = std::string(stringize(WASM_OBJDUMP_PATH)) +
                            " -x -r -d ../src/linker/linked_my.wasm "
                            ">../src/linker/linked_my.dump";
    system(objdumpCommand.c_str());
    system("cp mods_build/linked.wasm ../src/linker/linked_lld.wasm");
    system("cp mods_build/linked.wasm ../src/linker/linked_lld.wasm");
    objdumpCommand = std::string(stringize(WASM_OBJDUMP_PATH)) +
                     " -x -r -d ../src/linker/linked_lld.wasm "
                     ">../src/linker/linked_lld.dump";
    system(objdumpCommand.c_str());
    system("wasm-strip ../src/linker/linked_lld.wasm -o "
           "../src/linker/linked_lld_stripped.wasm");
    objdumpCommand = std::string(stringize(WASM_OBJDUMP_PATH)) +
                     " -x -r -d ../src/linker/linked_lld_stripped.wasm "
                     ">../src/linker/linked_lld_stripped.dump";
    system(objdumpCommand.c_str());
    system("diff ../src/linker/linked_my.dump "
           "../src/linker/linked_lld_stripped.dump >../src/linker/linked.diff");
    return Result::Ok;
}

int main(int argc, char *argv[]) {
    if (failed(run())) {
        return 1;
    }
    return 0;
}