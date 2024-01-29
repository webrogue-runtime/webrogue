#pragma once

#include "StringRef.h"
#include "base-types.h"
#include "cast.h"
#include "common.h"
#include "ir.h"
#include "leb128.h"
#include "multikey_sort.h"
#include "result.h"
#include "stream.h"
#include "synthesizer/function-synthesizer.h"
#include "synthesizer/import-synthesizer.h"
#include "type.h"
#include "xxhash.h"
#include <algorithm>
#include <cassert>
#include <cstdint>
#include <cstring>
#include <list>
#include <map>
#include <memory>
#include <set>
#include <string>
#include <utility>
#include <vector>

using namespace std;
namespace wabt {

inline Address aling(Address minValue, Address wordSize) {
    return (((minValue - 1) / wordSize) + 1) * wordSize;
}

enum RelocSemantic {
    FuncIndex,
    TableIndex,
    MemoryAddress,
    TypeIndex,
    GlobalIndex,
    FunctionOffset,
    SectionOffset,
    TagIndex,
    MemoryAddressRel,
    TableNumber,
    TableIndexRel,
    MemoryAddressTLS,
};

inline RelocSemantic getRelocSemantic(RelocType relocType) {
    switch (relocType) {
    case RelocType::FuncIndexLEB:
        return RelocSemantic::FuncIndex;
    case RelocType::TableIndexSLEB:
        return RelocSemantic::TableIndex;
    case RelocType::MemoryAddressLEB:
        return RelocSemantic::MemoryAddress;
    case RelocType::MemoryAddressSLEB:
        return RelocSemantic::MemoryAddress;
    case RelocType::MemoryAddressI32:
        return RelocSemantic::MemoryAddress;
    case RelocType::TypeIndexLEB:
        return RelocSemantic::TypeIndex;
    case RelocType::GlobalIndexLEB:
        return RelocSemantic::GlobalIndex;
    case RelocType::FunctionOffsetI32:
        return RelocSemantic::FunctionOffset;
    case RelocType::SectionOffsetI32:
        return RelocSemantic::SectionOffset;
    case RelocType::TagIndexLEB:
        return RelocSemantic::TagIndex;
    case RelocType::MemoryAddressRelSLEB:
        return RelocSemantic::MemoryAddressRel;
    case RelocType::TableIndexRelSLEB:
        return RelocSemantic::TableIndexRel;
    case RelocType::GlobalIndexI32:
        return RelocSemantic::GlobalIndex;
    case RelocType::MemoryAddressLEB64:
        return RelocSemantic::MemoryAddress;
    case RelocType::MemoryAddressSLEB64:
        return RelocSemantic::MemoryAddress;
    case RelocType::MemoryAddressI64:
        return RelocSemantic::MemoryAddress;
    case RelocType::MemoryAddressRelSLEB64:
        return RelocSemantic::MemoryAddressRel;
    case RelocType::TableIndexSLEB64:
        return RelocSemantic::TableIndex;
    case RelocType::TableIndexI64:
        return RelocSemantic::TableIndex;
    case RelocType::TableNumberLEB:
        return RelocSemantic::TableNumber;
    case RelocType::MemoryAddressTLSSLEB:
        return RelocSemantic::MemoryAddressTLS;
    case RelocType::MemoryAddressTLSI32:
        return RelocSemantic::MemoryAddressTLS;
    case RelocType::TableIndexI32:
        return RelocSemantic::TableIndex;
    }
    abort();
}

enum RelocForm {
    LEB,
    SLEB,
    I32,
    I64,
    LEB64,
    SLEB64,
};

struct ExtendedDataSegment : DataSegment {
    explicit ExtendedDataSegment(StringRef name) : DataSegment(name) {
    }

    map<string, set<pair<const WASMModule *, Offset>>> relativeSymbolOffsets;

    bool bss = false;
};

enum RelocSectionKind {
    Code,
    Data,
    Custom,
};

inline RelocForm getRelocForm(RelocType relocType) {
    switch (relocType) {
    case RelocType::FuncIndexLEB:
        return RelocForm::LEB;
    case RelocType::TableIndexSLEB:
        return RelocForm::SLEB;
    case RelocType::MemoryAddressLEB:
        return RelocForm::LEB;
    case RelocType::MemoryAddressSLEB:
        return RelocForm::SLEB;
    case RelocType::MemoryAddressI32:
        return RelocForm::I32;
    case RelocType::TypeIndexLEB:
        return RelocForm::LEB;
    case RelocType::GlobalIndexLEB:
        return RelocForm::LEB;
    case RelocType::FunctionOffsetI32:
        return RelocForm::I32;
    case RelocType::SectionOffsetI32:
        return RelocForm::I32;
    case RelocType::TagIndexLEB:
        return RelocForm::LEB;
    case RelocType::MemoryAddressRelSLEB:
        return RelocForm::SLEB;
    case RelocType::TableIndexRelSLEB:
        return RelocForm::SLEB;
    case RelocType::GlobalIndexI32:
        return RelocForm::I32;
    case RelocType::MemoryAddressLEB64:
        return RelocForm::LEB64;
    case RelocType::MemoryAddressSLEB64:
        return RelocForm::SLEB64;
    case RelocType::MemoryAddressI64:
        return RelocForm::I64;
    case RelocType::MemoryAddressRelSLEB64:
        return RelocForm::SLEB64;
    case RelocType::TableIndexSLEB64:
        return RelocForm::SLEB64;
    case RelocType::TableIndexI64:
        return RelocForm::I64;
    case RelocType::TableNumberLEB:
        return RelocForm::LEB;
    case RelocType::MemoryAddressTLSSLEB:
        return RelocForm::SLEB;
    case RelocType::MemoryAddressTLSI32:
        return RelocForm::I32;
    case RelocType::TableIndexI32:
        return RelocForm::I32;
    }
    abort();
}

class Synthesizer {
public:
    ImportSynthesizer importSynthesizer;
    FunctionSynthesizer functionSynthesizer;

    vector<unique_ptr<WASMModule>> *inputs;

    std::function<void()> interrupt = []() {
    };

    Address stackSize = 700000;
    const Address pageSize = 64 * 1024;
    Address totalPages;

    vector<FuncSignature> signatures;

    vector<Index> indirectFuncTable;

    map<string, WASMModule::Symbol *> symbols;

    vector<ImplementedFunc> implementedFuncs;
    set<string> weakFuncNames;

    vector<ExtendedDataSegment> data;

    map<pair<const WASMModule *, Index>, Index> dataRelocations;

    list<string> requiredSymbols;

    Address stackStart;
    Address stackEnd;
    Address dataStart;
    Address dataEnd;
    Address heapStart;
    Address heapEnd;

    void appendSignature(FuncSignature signature);

    Result synthesizeSignatures();

    Result synthesizeExports();

    Result relocateTable(Index originalIndex, const WASMModule *origin,
                         Index *outIndex);
    Result relocateType(Index originalIndex, const WASMModule *origin,
                        Index *outIndex);
    Result relocateFunc(Index originalIndex, const WASMModule *origin,
                        Index *outIndex);
    Result relocateTableElement(Index originalIndex, const WASMModule *origin,
                                Index *outIndex);
    Result relocateMemory(Index originalIndex, const WASMModule *origin,
                          Address *out);

    Result synthesizeTables();

    void generalizeDataSymbolName(StringRef &name);

    Result synthesizeData();

    Result relocateOne(RelocSemantic relocSemantic, WASMModule::Reloc reloc,
                       const WASMModule *origin, Address *out);

    Result writeReloc(RelocForm relocForm, uint8_t *dataPtr, uint8_t *end,
                      Address value);

    Result relocate();

    Result synthesize(std::vector<unique_ptr<WASMModule>> *inputs,
                      map<string, WASMModule::Symbol *> *symbols,
                      WASMModule *outputModule);
};
} // namespace wabt