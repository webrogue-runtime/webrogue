#include "compact-linker.h"
#include "archive-reader.h"
#include "synthesizer.h"
#include "write_file.h"
#include <iostream>

vector<uint8_t> compact_link(vector<string> requiredFunctions,
                             vector<LinkableFile> files,
                             std::function<void()> interrupt) {
    wabt::ArchiveReader archiveReader;
    archiveReader.interrupt = interrupt;

    for (auto requiredFunction : requiredFunctions)
        archiveReader.addRequiredFunction(requiredFunction.c_str());

    for (auto &file : files) {
        if (!wabt::succeeded(
                archiveReader.addFileData(&file.data, file.filename.c_str())))
            return {};
    }

    if (!wabt::succeeded(archiveReader.synthesize()))
        return {};
    wabt::Synthesizer synthesizer;
    synthesizer.interrupt = interrupt;
    synthesizer.stackSize = 5242880;
    wabt::WASMModule outputModule;
    if (!wabt::succeeded(synthesizer.synthesize(
            &archiveReader.modules, &archiveReader.symbols, &outputModule)))
        return {};
    wabt::MemoryStream stream;
    // // FileStream stream{};
    if (!wabt::succeeded(write_file(&outputModule, &stream)))
        return {};
    stream.Flush();
    return stream.output_buffer().data;
}