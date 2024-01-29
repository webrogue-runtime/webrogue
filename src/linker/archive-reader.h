#pragma once

#include "StringRef.h"
#include "base-types.h"
#include "binary-reader-ir.h"
#include "binary-reader.h"
#include "cast.h"
#include "ir.h"
#include "result.h"
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <list>
#include <map>
#include <memory>
#include <set>
#include <string>
#include <utility>
#include <vector>

using namespace std;

namespace wabt {
inline uint32_t parseArNumber(const char *str, size_t size, uint8_t base) {
    uint32_t result = 0;
    for (size_t i = 0; i < size; i++) {
        if (str[i] == 0x20)
            break;
        result = result * base + (str[i] - 0x30);
    }
    return result;
}

struct ArchiveOffsetInfo {
    Offset fileOffset;
    Offset fileSize;
    StringRef filename;
    bool isLoaded;
};

struct Archive;

class LazySymbol : public WASMModule::Symbol {
public:
    LazySymbol(std::string name, Index offset, Archive *archive)
        : Symbol(name, nullptr, true, false, false), offset(offset),
          archive(archive) {
    }

    Kind kind() const override {
        return Kind::Lazy;
    }
    static bool classof(const WASMModule::Symbol *entry) {
        return entry->kind() == WASMModule::Symbol::Kind::Lazy;
    }

    Index offset;
    Archive *archive;
};

struct Archive {
    vector<uint8_t> data;
    set<Offset> visitedOffsets;
    map<Offset, ArchiveOffsetInfo> offsetInfo;
    map<string, string> filenameMap;
    StringRef filename;
    vector<LazySymbol> symbols;
};

class ArchiveReader {

public:
    std::function<void()> interrupt = []() {
    };

    vector<unique_ptr<Archive>> archives;
    vector<vector<uint8_t>> moduleData;
    vector<unique_ptr<WASMModule>> modules;

    Index currentLoadOrder = 0;

    list<unique_ptr<WASMModule::FuncSymbol>> requiredSymbols;

    map<string, WASMModule::Symbol *> symbols;

    void addRequiredFunction(const char *funcName);

    Index findModule(WASMModule *wasmModule);
    Result addFile(const char *filename);

    Result addFileData(const std::vector<uint8_t> *data, const char *filename);

    set<string> comdats;
    Result addObjectFile(const vector<uint8_t> *data, StringRef filename);

    Result addArchiveFile(const vector<uint8_t> *data, StringRef filename);

    Result getFile(Archive *archive, Offset offset, Offset *outFileOffset,
                   Offset *outFileSize, StringRef *outFileName);
    Result readFilenameSection(Archive *archive, Offset offset, Offset size);

    Result readSymbolSection(Archive *archive, Offset offset, Offset size);

    WASMModule::Symbol *getSymbol(string name);

    Result loadLazySymbol(LazySymbol *symbol);

    Result insertSymbol(WASMModule::Symbol *symbol,
                        set<string> const &ignoredComdats);

    Result synthesize();
};
} // namespace wabt
