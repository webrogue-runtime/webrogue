#include "archive-reader.h"
#include "stream.h"
#include "synthesizer.h"
#include "write_file.h"

#include <cstdlib>
#include <string>

#define xstr(s) str(s)
#define str(s) #s

using namespace wabt;
Result run() {
    // system("pwd");
    ArchiveReader archiveReader;
    archiveReader.addRequiredFunction("init_mod_log2048");
    archiveReader.addRequiredFunction("init_mod_core");
    archiveReader.addRequiredFunction("wr_start");
    system("rm -f actual");
    CHECK_RESULT(archiveReader.addFile("../mods/log2048/mod.a"));
    CHECK_RESULT(archiveReader.addFile("../mods/core/mod.a"));
    CHECK_RESULT(archiveReader.addFile("../mods/core/stdlibs.a"));
    archiveReader.synthesize();
    Synthesizer synthesizer;
    WASMModule outputModule;
    auto symbols = archiveReader.symbols;
    CHECK_RESULT(synthesizer.synthesize(&archiveReader.modules,
                                        &archiveReader.symbols, &outputModule));
    MemoryStream stream;
    // // FileStream stream{};
    CHECK_RESULT(write_file(&outputModule, &stream));
    stream.Flush();
    stream.WriteToFile("../src/linker/test_src/t1_my.wasm");

    string const objdumpCommand = std::string(xstr(WASM_OBJDUMP_PATH)) +
                                  " -x -r -d ../src/linker/test_src/t1_my.wasm "
                                  ">../src/linker/test_src/t1_my.dump";
    system(objdumpCommand.c_str());
    return Result::Ok;
}

int main(int argc, char *argv[]) {
    if (failed(run())) {
        return 1;
    }
    return 0;
}