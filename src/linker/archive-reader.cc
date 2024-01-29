#include "archive-reader.h"
using namespace std;

namespace wabt {
void ArchiveReader::addRequiredFunction(const char *funcName) {
    unique_ptr<WASMModule::FuncSymbol> symbol =
        make_unique<WASMModule::FuncSymbol>(funcName, nullptr, -1000, false,
                                            false, false);
    // symbols[func_name] = symbol.get();
    requiredSymbols.push_back(std::move(symbol));
}
Index ArchiveReader::findModule(WASMModule *wasmModule) {
    for (Index i = 0; i < modules.size(); i++)
        if (modules[i].get() == wasmModule)
            return i;
    return -1;
}
Result ArchiveReader::addFile(const char *filename) {
    std::vector<uint8_t> data;
    CHECK_RESULT(ReadFile(filename, &data));
    if (data[0] == '\0' && data[1] == 'a' && data[2] == 's' && data[3] == 'm')
        CHECK_RESULT(addObjectFile(&data, filename));
    else
        CHECK_RESULT(addArchiveFile(&data, filename));

    return Result::Ok;
}
Result ArchiveReader::addFileData(const std::vector<uint8_t> *data,
                                  const char *filename) {
    if (data->at(0) == '\0' && data->at(1) == 'a' && data->at(2) == 's' &&
        data->at(3) == 'm') {
        CHECK_RESULT(addObjectFile(data, filename));
    } else {
        CHECK_RESULT(addArchiveFile(data, filename));
    }
    return Result::Ok;
}
Result ArchiveReader::addObjectFile(const vector<uint8_t> *data,
                                    StringRef filename) {
    set<string> ignoredComdats;
    moduleData.push_back(*data);
    vector<uint8_t> &pushedData = *moduleData.rbegin();
    auto module = make_unique<WASMModule>();
    ReadBinaryOptions readOptions;
    readOptions.readDebugNames = true;

    string filenameStr = filename.str();
    module->name = filename.str();
    CHECK_RESULT(readBinaryIr(filenameStr.c_str(), pushedData.data(),
                              pushedData.size(), readOptions, {},
                              module.get()));
    // {
    //   string command = "echo \'" + string(filename) + "\' >> actual";
    //   cout << filename << "\n";
    //   system(command.c_str());
    // }
    for (auto &symbol : module->symbols) {
        if (symbol->kind() == WASMModule::Symbol::Kind::Func)
            for (string comdat :
                 cast<WASMModule::FuncSymbol>(symbol.get())->comdats) {
                if (comdats.count(comdat))
                    ignoredComdats.emplace(comdat);
                else
                    comdats.emplace(comdat);
            }
        if (symbol->kind() == WASMModule::Symbol::Kind::Data)
            for (string comdat :
                 cast<WASMModule::DataSymbol>(symbol.get())->comdats) {
                if (comdats.count(comdat)) {
                    cast<WASMModule::DataSymbol>(symbol.get())
                        ->ignored_by_comdat = true;
                    ignoredComdats.emplace(comdat);
                } else
                    comdats.emplace(comdat);
            }
    }
    for (auto &symbol : module->symbols) {
        CHECK_RESULT(insertSymbol(symbol.get(), ignoredComdats));
    }
    modules.push_back(std::move(module));
    return Result::Ok;
}
Result ArchiveReader::addArchiveFile(const vector<uint8_t> *data,
                                     StringRef filename) {
    set<string> ignoredComdats;
    archives.push_back(make_unique<Archive>());
    Archive *archive = archives.rbegin()->get();
    archive->data = *data;
    archive->filename = filename;
    Offset offset = 7;
    while (offset < archive->data.size() - 60) {
        while (archive->data[offset] == 0x0A)
            offset++;
        Offset fileOffset;
        Offset fileSize;
        StringRef filename;
        CHECK_RESULT(
            getFile(archive, offset, &fileOffset, &fileSize, &filename));

        if (filename == "/") {
            CHECK_RESULT(readSymbolSection(archive, fileOffset, fileSize));
        } else if (filename == "//") {
            CHECK_RESULT(readFilenameSection(archive, fileOffset, fileSize));
        } else {
            ArchiveOffsetInfo offsetInfo;
            offsetInfo.fileOffset = fileOffset;
            offsetInfo.fileSize = fileSize;
            if (archive->filenameMap.count(filename.str()))
                offsetInfo.filename = archive->filenameMap[filename.str()];
            else
                offsetInfo.filename = filename;
            offsetInfo.filename =
                offsetInfo.filename.substr(0, offsetInfo.filename.size() - 1);
            offsetInfo.isLoaded = false;
            archive->offsetInfo[offset] = offsetInfo;
        }
        offset = fileOffset + fileSize;
    }
    for (auto symbolIter = archive->symbols.begin();
         symbolIter < archive->symbols.end(); symbolIter++) {
        insertSymbol(&*symbolIter, ignoredComdats);
    }
    return Result::Ok;
}
Result ArchiveReader::getFile(Archive *archive, Offset offset,
                              Offset *outFileOffset, Offset *outFileSize,
                              StringRef *outFileName) {
    Offset cursor = offset;
    vector<uint8_t> const &data = archive->data;

    StringRef filename = {reinterpret_cast<const char *>(data.data() + cursor),
                          16};
    cursor += 16;

    while (filename.size() && filename[filename.size() - 1] == ' ') {
        filename = filename.substr(0, filename.size() - 1);
    }

    char rawTimestamp[12];
    memcpy(rawTimestamp, data.data() + cursor, 12);
    cursor += 12;

    char rawOwnerId[6];
    memcpy(rawOwnerId, data.data() + cursor, 6);
    cursor += 6;

    char rawGroupId[6];
    memcpy(rawGroupId, data.data() + cursor, 6);
    cursor += 6;

    char rawFilemode[8];
    memcpy(rawFilemode, data.data() + cursor, 8);
    cursor += 8;

    char rawFilesize[10];
    memcpy(rawFilesize, data.data() + cursor, 10);
    cursor += 10;

    uint32_t filesize = parseArNumber(rawFilesize, 10, 10);

    if (data[cursor++] != 0x60)
        return Result::Error;
    if (data[cursor++] != 0x0A)
        return Result::Error;
    *outFileOffset = cursor;
    *outFileSize = filesize;
    *outFileName = filename;
    return Result::Ok;
};
Result ArchiveReader::readFilenameSection(Archive *archive, Offset offset,
                                          Offset size) {
    Offset cursor = offset;
    vector<uint8_t> const &data = archive->data;
    Offset lastStart = offset;
    while (cursor < offset + size) {
        if (data[cursor] == 0x0A) {
            interrupt();
            StringRef filename = {
                reinterpret_cast<const char *>(data.data() + lastStart),
                cursor - lastStart};
            string shortenName = "/" + to_string(lastStart - offset);
            archive->filenameMap[shortenName] = filename.str();
            lastStart = cursor + 1;
        }
        cursor++;
    }
    return Result::Ok;
}
Result ArchiveReader::readSymbolSection(Archive *archive, Offset offset,
                                        Offset size) {
    Offset cursor = offset;
    vector<uint8_t> const &data = archive->data;
    auto readInt32 = [data, &cursor](Index &out) {
        out = (data[cursor + 0] << 24) + (data[cursor + 1] << 16) +
              (data[cursor + 2] << 8) + (data[cursor + 3] << 0);
        cursor += 4;
    };
    Index symbolCount;
    readInt32(symbolCount);
    archive->symbols.clear();
    archive->symbols.reserve(symbolCount);
    vector<Index> offsets;
    for (Index symbolIndex = 0; symbolIndex < symbolCount; symbolIndex++) {
        interrupt();
        Index offset;
        readInt32(offset);
        offsets.push_back(offset);
    }
    for (Index symbolIndex = 0; symbolIndex < symbolCount; symbolIndex++) {
        interrupt();
        size_t nameLen =
            strlen(reinterpret_cast<const char *>(data.data() + cursor));
        StringRef name = {reinterpret_cast<const char *>(data.data() + cursor),
                          nameLen};
        archive->symbols.push_back(
            LazySymbol(name.str(), offsets[symbolIndex], archive));
        cursor += nameLen + 1;
    }
    return Result::Ok;
}
WASMModule::Symbol *ArchiveReader::getSymbol(string name) {
    if (symbols.count(name))
        return symbols[name];
    return nullptr;
}
Result ArchiveReader::loadLazySymbol(LazySymbol *symbol) {
    interrupt();
    ArchiveOffsetInfo &offsetInfo = symbol->archive->offsetInfo[symbol->offset];
    if (offsetInfo.isLoaded)
        return Result::Ok;
    uint8_t *fileStart = symbol->archive->data.data() + offsetInfo.fileOffset;
    vector<uint8_t> fileData = {fileStart, fileStart + offsetInfo.fileSize};
    string filename =
        symbol->archive->filename.str() + "(" + offsetInfo.filename.str() + ")";
    offsetInfo.isLoaded = true;
    CHECK_RESULT(addObjectFile(&fileData, filename));
    return Result::Ok;
}

Result ArchiveReader::insertSymbol(WASMModule::Symbol *symbol,
                                   set<string> const &ignoredComdats) {
    interrupt();
    assert(symbol);
    WASMModule::Symbol *oldSymbol = getSymbol(symbol->name);
    auto setSymbol = [this, symbol]() {
        symbols[symbol->name] = symbol;
    };
    bool hasComdatCollision = false;
    if (symbol->kind() == WASMModule::Symbol::Kind::Func) {
        hasComdatCollision = ignoredComdats.count(symbol->name);
    }
    Index loadOrder;
    if (oldSymbol)
        loadOrder = oldSymbol->load_order = currentLoadOrder;
    else
        loadOrder = ++currentLoadOrder;
    symbol->load_order = loadOrder;
    if (oldSymbol == nullptr) {
        setSymbol();
        return Result::Ok;
    }
    if (symbol->kind() == WASMModule::Symbol::Kind::Lazy) {
        if (!oldSymbol->defined) {
            CHECK_RESULT(loadLazySymbol(cast<LazySymbol>(symbol)));
        } else {
            // assert(false); // weak?
        }
        return Result::Ok;
    }
    if (!symbol->defined &&
        oldSymbol->kind() == WASMModule::Symbol::Kind::Lazy) {
        if (symbol->weak) {
            oldSymbol->weak = true;
        } else {
            CHECK_RESULT(loadLazySymbol(cast<LazySymbol>(oldSymbol)));
        }
        return Result::Ok;
    }
    if (!symbol->defined)
        return Result::Ok;
    if (oldSymbol->kind() == WASMModule::Symbol::Kind::Lazy) {
        setSymbol();
        return Result::Ok;
    }
    if (!oldSymbol->defined) {
        setSymbol();
        return Result::Ok;
    }
    if (oldSymbol->weak && symbol->weak) {
        if (!hasComdatCollision)
            setSymbol();
        return Result::Ok;
    }
    if (!oldSymbol->defined || (oldSymbol->weak && !symbol->weak)) {
        setSymbol();
        return Result::Ok;
    }

    return Result::Ok;
};

Result ArchiveReader::synthesize() {
    set<string> remainLazySymbolNames;
    for (auto &requiredSymbol : requiredSymbols) {
        if (symbols[requiredSymbol->name]->kind() ==
            WASMModule::Symbol::Kind::Lazy)
            loadLazySymbol(cast<LazySymbol>(symbols[requiredSymbol->name]));
        symbols[requiredSymbol->name]->exported = true;
    }
    for (auto symbolIter : symbols)
        if (symbolIter.second->kind() == WASMModule::Symbol::Kind::Lazy)
            remainLazySymbolNames.emplace(symbolIter.first);
    for (auto lazySymbolName : remainLazySymbolNames) {
        if (symbols[lazySymbolName]->weak) {
            for (auto &module : modules)
                for (auto &symbol : module->symbols)
                    if (symbol->name == lazySymbolName)
                        symbols[lazySymbolName] = symbol.get();
        } else
            symbols.erase(lazySymbolName);
    }

    return Result::Ok;
}

} // namespace wabt
