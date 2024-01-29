/*
 * Copyright 2016 WebAssembly Community Group participants
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include "binary-reader-ir.h"

#include <cassert>
#include <cstdint>
#include <cstdio>
#include <deque>
#include <string>
#include <utility>
#include <vector>

#include "base-types.h"
#include "binary-reader.h"
#include "binary.h"
#include "cast.h"
#include "common.h"
#include "ir.h"
#include "result.h"

namespace wabt {

namespace {

struct LabelNode {
    LabelNode(LabelType, ExprList *exprs, Expr *context = nullptr);

    LabelType labelType;
    ExprList *exprs;
    Expr *context;
};

LabelNode::LabelNode(LabelType labelType, ExprList *exprs, Expr *context)
    : labelType(labelType), exprs(exprs), context(context) {
}

class CodeMetadataExprQueue {
private:
    struct Entry {
        Func *func;
        std::deque<std::unique_ptr<CodeMetadataExpr>> funcQueue;
        Entry(Func *f) : func(f) {
        }
    };
    std::deque<Entry> entries;

public:
    CodeMetadataExprQueue() {
    }
    void pushFunc(Func *f) {
        entries.emplace_back(f);
    }
    void pushMetadata(std::unique_ptr<CodeMetadataExpr> meta) {
        assert(!entries.empty());
        entries.back().funcQueue.push_back(std::move(meta));
    }

    std::unique_ptr<CodeMetadataExpr> popMatch(Func *f, Offset offset) {
        std::unique_ptr<CodeMetadataExpr> ret;
        if (entries.empty()) {
            return ret;
        }

        auto &currentEntry = entries.front();

        if (currentEntry.func != f)
            return ret;
        if (currentEntry.funcQueue.empty()) {
            entries.pop_front();
            return ret;
        }

        auto &currentMetadata = currentEntry.funcQueue.front();
        ret = std::move(currentMetadata);
        currentEntry.funcQueue.pop_front();

        return ret;
    }
};

class BinaryReaderIR : public BinaryReaderDelegate {
    static constexpr size_t kMaxNestingDepth =
        16384; // max depth of label stack
    static constexpr size_t kMaxFunctionLocals = 50000; // matches V8
    static constexpr size_t kMaxFunctionParams = 1000;  // matches V8
    static constexpr size_t kMaxFunctionResults = 1000; // matches V8

public:
    BinaryReaderIR(WASMModule *outModule, const char *filename, Errors *errors);

    Result BeginModule(uint32_t version) override {
        return Result::Ok;
    }
    Result EndModule() override {
        return Result::Ok;
    }

    Result BeginCustomSection(Index sectionIndex, Offset size,
                              StringRef sectionName) override {
        return Result::Ok;
    }
    Result EndCustomSection() override {
        return Result::Ok;
    }

    Result BeginTypeSection(Offset size) override {
        return Result::Ok;
    }
    Result EndTypeSection() override {
        return Result::Ok;
    }
    Result BeginImportSection(Offset size) override {
        return Result::Ok;
    }
    Result OnImport(Index index, ExternalKind kind, StringRef moduleName,
                    StringRef fieldName) override {
        return Result::Ok;
    }
    Result EndImportSection() override {
        return Result::Ok;
    }
    Result BeginFunctionSection(Offset size) override {
        return Result::Ok;
    }
    Result EndFunctionSection() override {
        return Result::Ok;
    }
    Result BeginTableSection(Offset size) override {
        return Result::Ok;
    }
    Result EndTableSection() override {
        return Result::Ok;
    }
    Result BeginMemorySection(Offset size) override {
        return Result::Ok;
    }
    Result EndMemorySection() override {
        return Result::Ok;
    }
    Result BeginGlobalSection(Offset size) override {
        return Result::Ok;
    }
    Result EndGlobal(Index index) override {
        return Result::Ok;
    }
    Result EndGlobalSection() override {
        return Result::Ok;
    }
    Result BeginExportSection(Offset size) override {
        return Result::Ok;
    }
    Result EndExportSection() override {
        return Result::Ok;
    }
    Result BeginStartSection(Offset size) override {
        return Result::Ok;
    }
    Result EndStartSection() override {
        return Result::Ok;
    }
    Result OnLocalDeclCount(Index count) override {
        return Result::Ok;
    }
    Result OnOpcodeBare() override {
        return Result::Ok;
    }
    Result OnOpcodeUint32(uint32_t value) override {
        return Result::Ok;
    }
    Result OnOpcodeIndex(Index value) override {
        return Result::Ok;
    }
    Result OnOpcodeIndexIndex(Index value, Index value2) override {
        return Result::Ok;
    }
    Result OnOpcodeUint32Uint32(uint32_t value, uint32_t value2) override {
        return Result::Ok;
    }
    Result OnOpcodeUint32Uint32Uint32(uint32_t value, uint32_t value2,
                                      uint32_t value3) override {
        return Result::Ok;
    }
    Result OnOpcodeUint32Uint32Uint32Uint32(uint32_t value, uint32_t value2,
                                            uint32_t value3,
                                            uint32_t value4) override {
        return Result::Ok;
    }
    Result OnOpcodeUint64(uint64_t value) override {
        return Result::Ok;
    }
    Result OnOpcodeF32(uint32_t value) override {
        return Result::Ok;
    }
    Result OnOpcodeF64(uint64_t value) override {
        return Result::Ok;
    }
    Result OnOpcodeV128(v128 value) override {
        return Result::Ok;
    }
    Result OnOpcodeBlockSig(Type sigType) override {
        return Result::Ok;
    }
    Result OnOpcodeType(Type type) override {
        return Result::Ok;
    }
    Result EndCodeSection() override {
        return Result::Ok;
    }
    Result BeginElemSection(Offset size) override {
        return Result::Ok;
    }
    Result EndElemSection() override {
        return Result::Ok;
    }
    Result EndElemSegment(Index index) override {
        return Result::Ok;
    }
    Result EndDataSegment(Index index) override {
        return Result::Ok;
    }
    Result EndDataSection() override {
        return Result::Ok;
    }
    Result BeginDataCountSection(Offset size) override {
        return Result::Ok;
    }
    Result OnDataCount(Index count) override {
        return Result::Ok;
    }
    Result EndDataCountSection() override {
        return Result::Ok;
    }
    Result BeginNamesSection(Offset size) override {
        return Result::Ok;
    }
    Result OnModuleNameSubsection(Index index, uint32_t nameType,
                                  Offset subsectionSize) override {
        return Result::Ok;
    }
    Result OnFunctionNameSubsection(Index index, uint32_t nameType,
                                    Offset subsectionSize) override {
        return Result::Ok;
    }
    Result OnLocalNameSubsection(Index index, uint32_t nameType,
                                 Offset subsectionSize) override {
        return Result::Ok;
    }
    Result OnLocalNameFunctionCount(Index numFunctions) override {
        return Result::Ok;
    }
    Result OnNameSubsection(Index index, NameSectionSubsection subsectionType,
                            Offset subsectionSize) override {
        return Result::Ok;
    }
    Result OnNameCount(Index numNames) override {
        return Result::Ok;
    }
    Result EndNamesSection() override {
        return Result::Ok;
    }
    Result BeginRelocSection(Offset size) override {
        return Result::Ok;
    }
    Result EndRelocSection() override {
        return Result::Ok;
    }
    Result BeginDylinkSection(Offset size) override {
        return Result::Ok;
    }
    Result OnDylinkInfo(uint32_t memSize, uint32_t memAlign, uint32_t tableSize,
                        uint32_t tableAlign) override {
        return Result::Ok;
    }
    Result OnDylinkNeededCount(Index count) override {
        return Result::Ok;
    }
    Result OnDylinkNeeded(StringRef soName) override {
        return Result::Ok;
    }
    Result OnDylinkImportCount(Index count) override {
        return Result::Ok;
    }
    Result OnDylinkExportCount(Index count) override {
        return Result::Ok;
    }
    Result OnDylinkImport(StringRef module, StringRef name,
                          uint32_t flags) override {
        return Result::Ok;
    }
    Result OnDylinkExport(StringRef name, uint32_t flags) override {
        return Result::Ok;
    }
    Result EndDylinkSection() override {
        return Result::Ok;
    }
    Result BeginTargetFeaturesSection(Offset size) override {
        return Result::Ok;
    }
    Result OnFeatureCount(Index count) override {
        return Result::Ok;
    }
    Result OnFeature(uint8_t prefix, StringRef name) override {
        return Result::Ok;
    }
    Result EndTargetFeaturesSection() override {
        return Result::Ok;
    }
    Result BeginGenericCustomSection(Offset size) override {
        return Result::Ok;
    }
    Result EndGenericCustomSection() override {
        return Result::Ok;
    }
    Result BeginLinkingSection(Offset size) override {
        return Result::Ok;
    }
    Result OnSymbolCount(Index count) override {
        return Result::Ok;
    }
    Result OnSegmentInfoCount(Index count) override {
        return Result::Ok;
    }
    Result OnInitFunctionCount(Index count) override {
        return Result::Ok;
    }
    Result OnComdatCount(Index count) override {
        return Result::Ok;
    }
    Result EndLinkingSection() override {
        return Result::Ok;
    }
    Result EndCodeMetadataSection() override {
        return Result::Ok;
    }

    bool OnError(const Error &) override;

    Result OnTypeCount(Index count) override;
    Result OnFuncType(Index index, Index paramCount, Type *paramTypes,
                      Index resultCount, Type *resultTypes) override;
    Result OnStructType(Index index, Index fieldCount,
                        TypeMut *fields) override;
    Result OnArrayType(Index index, TypeMut field) override;

    Result OnImportCount(Index count) override;
    Result OnImportFunc(Index importIndex, StringRef moduleName,
                        StringRef fieldName, Index funcIndex,
                        Index sigIndex) override;
    Result OnImportTable(Index importIndex, StringRef moduleName,
                         StringRef fieldName, Index tableIndex, Type elemType,
                         const Limits *elemLimits) override;
    Result OnImportMemory(Index importIndex, StringRef moduleName,
                          StringRef fieldName, Index memoryIndex,
                          const Limits *pageLimits) override;
    Result OnImportGlobal(Index importIndex, StringRef moduleName,
                          StringRef fieldName, Index globalIndex, Type type,
                          bool isMutable) override;
    Result OnImportTag(Index importIndex, StringRef moduleName,
                       StringRef fieldName, Index tagIndex,
                       Index sigIndex) override;

    Result OnFunctionCount(Index count) override;
    Result OnFunction(Index index, Index sigIndex) override;

    Result OnTableCount(Index count) override;
    Result OnTable(Index index, Type elemType,
                   const Limits *elemLimits) override;

    Result OnMemoryCount(Index count) override;
    Result OnMemory(Index index, const Limits *limits) override;

    Result OnGlobalCount(Index count) override;
    Result BeginGlobal(Index index, Type type, bool isMutable) override;
    Result BeginGlobalInitExpr(Index index) override;
    Result EndGlobalInitExpr(Index index) override;

    Result OnExportCount(Index count) override;
    Result OnExport(Index index, ExternalKind kind, Index itemIndex,
                    StringRef name) override;

    Result OnStartFunction(Index funcIndex) override;

    Result OnFunctionBodyCount(Index count) override;
    Result BeginFunctionBody(Index index, Offset size) override;
    Result OnLocalDecl(Index declIndex, Index count, Type type) override;

    Result OnOpcode(Opcode opcode) override;
    Result OnAtomicLoadExpr(Opcode opcode, Index memidx, Address alignmentLog2,
                            Address offset) override;
    Result OnAtomicStoreExpr(Opcode opcode, Index memidx, Address alignmentLog2,
                             Address offset) override;
    Result OnAtomicRmwExpr(Opcode opcode, Index memidx, Address alignmentLog2,
                           Address offset) override;
    Result OnAtomicRmwCmpxchgExpr(Opcode opcode, Index memidx,
                                  Address alignmentLog2,
                                  Address offset) override;
    Result OnAtomicWaitExpr(Opcode opcode, Index memidx, Address alignmentLog2,
                            Address offset) override;
    Result OnAtomicFenceExpr(uint32_t consistencyModel) override;
    Result OnAtomicNotifyExpr(Opcode opcode, Index memidx,
                              Address alignmentLog2, Address offset) override;
    Result OnBinaryExpr(Opcode opcode) override;
    Result OnBlockExpr(Type sigType) override;
    Result OnBrExpr(Index depth) override;
    Result OnBrIfExpr(Index depth) override;
    Result OnBrTableExpr(Index numTargets, Index *targetDepths,
                         Index defaultTargetDepth) override;
    Result OnCallExpr(Index funcIndex) override;
    Result OnCatchExpr(Index tagIndex) override;
    Result OnCatchAllExpr() override;
    Result OnCallIndirectExpr(Index sigIndex, Index tableIndex) override;
    Result OnCallRefExpr() override;
    Result OnReturnCallExpr(Index funcIndex) override;
    Result OnReturnCallIndirectExpr(Index sigIndex, Index tableIndex) override;
    Result OnCompareExpr(Opcode opcode) override;
    Result OnConvertExpr(Opcode opcode) override;
    Result OnDelegateExpr(Index depth) override;
    Result OnDropExpr() override;
    Result OnElseExpr() override;
    Result OnEndExpr() override;
    Result OnF32ConstExpr(uint32_t valueBits) override;
    Result OnF64ConstExpr(uint64_t valueBits) override;
    Result OnV128ConstExpr(v128 valueBits) override;
    Result OnGlobalGetExpr(Index globalIndex) override;
    Result OnGlobalSetExpr(Index globalIndex) override;
    Result OnI32ConstExpr(uint32_t value) override;
    Result OnI64ConstExpr(uint64_t value) override;
    Result OnIfExpr(Type sigType) override;
    Result OnLoadExpr(Opcode opcode, Index memidx, Address alignmentLog2,
                      Address offset) override;
    Result OnLocalGetExpr(Index localIndex) override;
    Result OnLocalSetExpr(Index localIndex) override;
    Result OnLocalTeeExpr(Index localIndex) override;
    Result OnLoopExpr(Type sigType) override;
    Result OnMemoryCopyExpr(Index destmemidx, Index srcmemidx) override;
    Result OnDataDropExpr(Index segmentIndex) override;
    Result OnMemoryFillExpr(Index memidx) override;
    Result OnMemoryGrowExpr(Index memidx) override;
    Result OnMemoryInitExpr(Index segmentIndex, Index memidx) override;
    Result OnMemorySizeExpr(Index memidx) override;
    Result OnTableCopyExpr(Index dstIndex, Index srcIndex) override;
    Result OnElemDropExpr(Index segmentIndex) override;
    Result OnTableInitExpr(Index segmentIndex, Index tableIndex) override;
    Result OnTableGetExpr(Index tableIndex) override;
    Result OnTableSetExpr(Index tableIndex) override;
    Result OnTableGrowExpr(Index tableIndex) override;
    Result OnTableSizeExpr(Index tableIndex) override;
    Result OnTableFillExpr(Index tableIndex) override;
    Result OnRefFuncExpr(Index funcIndex) override;
    Result OnRefNullExpr(Type type) override;
    Result OnRefIsNullExpr() override;
    Result OnNopExpr() override;
    Result OnRethrowExpr(Index depth) override;
    Result OnReturnExpr() override;
    Result OnSelectExpr(Index resultCount, Type *resultTypes) override;
    Result OnStoreExpr(Opcode opcode, Index memidx, Address alignmentLog2,
                       Address offset) override;
    Result OnThrowExpr(Index tagIndex) override;
    Result OnTryExpr(Type sigType) override;
    Result OnUnaryExpr(Opcode opcode) override;
    Result OnTernaryExpr(Opcode opcode) override;
    Result OnUnreachableExpr() override;
    Result EndFunctionBody(Index index) override;
    Result OnSimdLaneOpExpr(Opcode opcode, uint64_t value) override;
    Result OnSimdLoadLaneExpr(Opcode opcode, Index memidx,
                              Address alignmentLog2, Address offset,
                              uint64_t value) override;
    Result OnSimdStoreLaneExpr(Opcode opcode, Index memidx,
                               Address alignmentLog2, Address offset,
                               uint64_t value) override;
    Result OnSimdShuffleOpExpr(Opcode opcode, v128 value) override;
    Result OnLoadSplatExpr(Opcode opcode, Index memidx, Address alignmentLog2,
                           Address offset) override;
    Result OnLoadZeroExpr(Opcode opcode, Index memidx, Address alignmentLog2,
                          Address offset) override;

    Result OnElemSegmentCount(Index count) override;
    Result BeginElemSegment(Index index, Index tableIndex,
                            uint8_t flags) override;
    Result BeginElemSegmentInitExpr(Index index) override;
    Result EndElemSegmentInitExpr(Index index) override;
    Result OnElemSegmentElemType(Index index, Type elemType) override;
    Result OnElemSegmentElemExprCount(Index index, Index count) override;
    Result BeginElemExpr(Index elemIndex, Index exprIndex) override;
    Result EndElemExpr(Index elemIndex, Index exprIndex) override;

    Result OnDataSegmentCount(Index count) override;
    Result BeginDataSegment(Index index, Index memoryIndex,
                            uint8_t flags) override;
    Result BeginDataSegmentInitExpr(Index index) override;
    Result EndDataSegmentInitExpr(Index index) override;
    Result OnDataSegmentData(Index index, const void *data,
                             Address size) override;

    Result OnModuleName(StringRef moduleName) override;
    Result OnFunctionNamesCount(Index numFunctions) override;
    Result OnFunctionName(Index functionIndex, StringRef functionName) override;
    Result OnLocalNameLocalCount(Index functionIndex, Index numLocals) override;
    Result OnLocalName(Index functionIndex, Index localIndex,
                       StringRef localName) override;
    Result OnNameEntry(NameSectionSubsection type, Index index,
                       StringRef name) override;

    Result OnGenericCustomSection(StringRef name, const void *data,
                                  Offset size) override;

    Result BeginTagSection(Offset size) override {
        return Result::Ok;
    }
    Result OnTagCount(Index count) override {
        return Result::Ok;
    }
    Result OnTagType(Index index, Index sigIndex) override;
    Result EndTagSection() override {
        return Result::Ok;
    }

    Result OnDataSymbol(Index index, uint32_t flags, StringRef name,
                        Index segment, uint32_t offset, uint32_t size) override;
    Result OnFunctionSymbol(Index index, uint32_t flags, StringRef name,
                            Index funcIndex) override;
    Result OnGlobalSymbol(Index index, uint32_t flags, StringRef name,
                          Index globalIndex) override;
    Result OnSectionSymbol(Index index, uint32_t flags,
                           Index sectionIndex) override;
    /* Code Metadata sections */
    Result BeginCodeMetadataSection(StringRef name, Offset size) override;
    Result OnCodeMetadataFuncCount(Index count) override;
    Result OnCodeMetadataCount(Index functionIndex, Index count) override;
    Result OnCodeMetadata(Offset offset, const void *data,
                          Address size) override;

    Result OnTagSymbol(Index index, uint32_t flags, StringRef name,
                       Index tagIndex) override;
    Result OnTableSymbol(Index index, uint32_t flags, StringRef name,
                         Index tableIndex) override;
    // added methods
    Index relocSectionIndex;
    Result OnReloc(RelocType type, Offset offset, Index index,
                   uint32_t addend) override {
        WASMModule::Reloc reloc;
        reloc.index = index;
        reloc.addend = addend;
        reloc.offset = offset;
        reloc.section_index = relocSectionIndex;
        reloc.type = type;
        wasmModule->relocs.push_back(reloc);

        return Result::Ok;
    }
    Result OnRelocCount(Index count, Index sectionIndex) override {
        relocSectionIndex = sectionIndex;
        wasmModule->relocs.reserve(wasmModule->relocs.size() + count);
        return Result::Ok;
    }
    Result BeginSection(Index sectionIndex, BinarySection sectionCode,
                        Offset size) override {
        if (sectionCode == BinarySection::Code) {
            wasmModule->code_section_index = sectionIndex;
        } else if (sectionCode == BinarySection::Data) {
            wasmModule->data_section_index = sectionIndex;
        }
        return Result::Ok;
    }
    Result OnSegmentInfo(Index index, StringRef name, Address alignmentLog2,
                         uint32_t flags) override {
        wasmModule->data_segment_info.push_back({});
        wasmModule->data_segment_info.rbegin()->strings_flag =
            flags & WABT_SEGMENT_FLAG_STRINGS;
        wasmModule->data_segment_info.rbegin()->name = name.str();
        wasmModule->data_segment_info.rbegin()->alignment_log2 = alignmentLog2;
        return Result::Ok;
    }

    Offset codeStart;
    Result BeginCodeSection(Offset size) override {
        codeStart = state->offset;
        wasmModule->code = {state->data + state->offset,
                            state->data + state->offset + size};
        return Result::Ok;
    }

    Offset dataStart;
    Result BeginDataSection(Offset size) override {
        dataStart = state->offset;
        return Result::Ok;
    }

    Result OnInitFunction(uint32_t priority, Index symbolIndex) override {
        wasmModule->init_fimctions.push_back(
            std::make_pair(priority, wasmModule->symbols[symbolIndex].get()));
        return Result::Ok;
    }

    StringRef comdatName;

    Result OnComdatBegin(StringRef name, uint32_t flags, Index count) override {
        comdatName = name;
        return Result::Ok;
    }

    Result OnComdatEntry(ComdatType kind, Index index) override {
        switch (kind) {
        case ComdatType::Data: {
            for (auto &symbol : wasmModule->symbols) {
                if (symbol->kind() != WASMModule::Symbol::Kind::Data)
                    continue;
                auto *dataSymbol = cast<WASMModule::DataSymbol>(symbol.get());
                if (dataSymbol->segment_index != index)
                    continue;
                dataSymbol->comdats.emplace(comdatName.str());
            }
            break;
        }
        case ComdatType::Function: {
            for (auto &symbol : wasmModule->symbols) {
                if (symbol->kind() != WASMModule::Symbol::Kind::Func)
                    continue;
                auto *funcSymbol = cast<WASMModule::FuncSymbol>(symbol.get());
                if (funcSymbol->original_func_index != index)
                    continue;
                funcSymbol->comdats.emplace(comdatName.str());
            }
            break;
        }
        }
        return Result::Ok;
    }

private:
    void printError(const char *format, ...);
    Result pushLabel(LabelType labelType, ExprList *first,
                     Expr *context = nullptr);
    Result beginInitExpr(ExprList *initExpr);
    Result endInitExpr();
    Result popLabel();
    Result getLabelAt(LabelNode **label, Index depth);
    Result topLabel(LabelNode **label);
    Result topLabelExpr(LabelNode **label, Expr **expr);
    Result appendExpr(std::unique_ptr<Expr> expr);
    Result appendCatch(Catch &&catchObj);
    void setFuncDeclaration(FuncDeclaration *decl, Var var);
    void setBlockDeclaration(BlockDeclaration *decl, Type sigType);
    Result setMemoryName(Index index, StringRef name);
    Result setTableName(Index index, StringRef name);
    Result setFunctionName(Index index, StringRef name);
    Result setTypeName(Index index, StringRef name);
    Result setGlobalName(Index index, StringRef name);
    Result setDataSegmentName(Index index, StringRef name);
    Result setElemSegmentName(Index index, StringRef name);
    Result setTagName(Index index, StringRef name);

    std::string getUniqueName(BindingHash *bindings,
                              const std::string &originalName);

    Errors *errors = nullptr;
    WASMModule *wasmModule = nullptr;

    Func *currentFunc = nullptr;
    std::vector<LabelNode> labelStack;
    const char *filename;

    CodeMetadataExprQueue codeMetadataQueue;
    StringRef currentMetadataName;
};

BinaryReaderIR::BinaryReaderIR(WASMModule *outModule, const char *filename,
                               Errors *errors)
    : errors(errors), wasmModule(outModule), filename(filename) {
}

void WABT_PRINTF_FORMAT(2, 3) BinaryReaderIR::printError(const char *format,
                                                         ...) {
}

Result BinaryReaderIR::pushLabel(LabelType labelType, ExprList *first,
                                 Expr *context) {
    if (labelStack.size() >= kMaxNestingDepth) {
        printError("label stack exceeds max nesting depth");
        return Result::Error;
    }
    labelStack.emplace_back(labelType, first, context);
    return Result::Ok;
}

Result BinaryReaderIR::popLabel() {
    if (labelStack.size() == 0) {
        printError("popping empty label stack");
        return Result::Error;
    }

    labelStack.pop_back();
    return Result::Ok;
}

Result BinaryReaderIR::getLabelAt(LabelNode **label, Index depth) {
    if (depth >= labelStack.size()) {
        return Result::Error;
    }

    *label = &labelStack[labelStack.size() - depth - 1];
    return Result::Ok;
}

Result BinaryReaderIR::topLabel(LabelNode **label) {
    return getLabelAt(label, 0);
}

Result BinaryReaderIR::topLabelExpr(LabelNode **label, Expr **expr) {
    CHECK_RESULT(topLabel(label));
    LabelNode *parentLabel;
    CHECK_RESULT(getLabelAt(&parentLabel, 1));
    if (parentLabel->exprs->empty()) {
        printError("TopLabelExpr: parent label has empty expr list");
        return Result::Error;
    }
    *expr = &parentLabel->exprs->back();
    return Result::Ok;
}

Result BinaryReaderIR::appendExpr(std::unique_ptr<Expr> expr) {
    LabelNode *label;
    CHECK_RESULT(topLabel(&label));
    label->exprs->push_back(std::move(expr));
    return Result::Ok;
}

void BinaryReaderIR::setFuncDeclaration(FuncDeclaration *decl, Var var) {
    decl->has_func_type = true;
    decl->type_var = var;
    if (auto *funcType = wasmModule->GetFuncType(var)) {
        decl->sig = funcType->sig;
    }
}

void BinaryReaderIR::setBlockDeclaration(BlockDeclaration *decl, Type sigType) {
    if (sigType.IsIndex()) {
        Index typeIndex = sigType.GetIndex();
        setFuncDeclaration(decl, Var(typeIndex));
    } else {
        decl->has_func_type = false;
        decl->sig.param_types.clear();
        decl->sig.result_types = sigType.GetInlineVector();
    }
}

std::string BinaryReaderIR::getUniqueName(BindingHash *bindings,
                                          const std::string &origName) {
    int counter = 1;
    std::string uniqueName = origName;
    while (bindings->count(uniqueName) != 0) {
        uniqueName = origName + "." + std::to_string(counter++);
    }
    return uniqueName;
}

bool BinaryReaderIR::OnError(const Error &error) {
    errors->push_back(error);
    return true;
}

Result BinaryReaderIR::OnTypeCount(Index count) {
    WABT_TRY
    wasmModule->types.reserve(count);
    WABT_CATCH_BAD_ALLOC
    return Result::Ok;
}

Result BinaryReaderIR::OnFuncType(Index index, Index paramCount,
                                  Type *paramTypes, Index resultCount,
                                  Type *resultTypes) {
    if (paramCount > kMaxFunctionParams) {
        printError("FuncType param count exceeds maximum value");
        return Result::Error;
    }

    if (resultCount > kMaxFunctionResults) {
        printError("FuncType result count exceeds maximum value");
        return Result::Error;
    }

    auto field = std::make_unique<TypeModuleField>();
    auto funcType = std::make_unique<FuncType>();
    funcType->sig.param_types.assign(paramTypes, paramTypes + paramCount);
    funcType->sig.result_types.assign(resultTypes, resultTypes + resultCount);

    wasmModule->features_used.simd |=
        std::any_of(funcType->sig.param_types.begin(),
                    funcType->sig.param_types.end(),
                    [](auto x) {
                        return x == Type::V128;
                    }) ||
        std::any_of(funcType->sig.result_types.begin(),
                    funcType->sig.result_types.end(), [](auto x) {
                        return x == Type::V128;
                    });

    field->type = std::move(funcType);
    wasmModule->AppendField(std::move(field));
    return Result::Ok;
}

Result BinaryReaderIR::OnStructType(Index index, Index fieldCount,
                                    TypeMut *fields) {
    auto field = std::make_unique<TypeModuleField>();
    auto structType = std::make_unique<StructType>();
    structType->fields.resize(fieldCount);
    for (Index i = 0; i < fieldCount; ++i) {
        structType->fields[i].type = fields[i].type;
        structType->fields[i].mutable_ = fields[i].isMutable;
        wasmModule->features_used.simd |= (fields[i].type == Type::V128);
    }
    field->type = std::move(structType);
    wasmModule->AppendField(std::move(field));
    return Result::Ok;
}

Result BinaryReaderIR::OnArrayType(Index index, TypeMut typeMut) {
    auto field = std::make_unique<TypeModuleField>();
    auto arrayType = std::make_unique<ArrayType>();
    arrayType->field.type = typeMut.type;
    arrayType->field.mutable_ = typeMut.isMutable;
    wasmModule->features_used.simd |= (typeMut.type == Type::V128);
    field->type = std::move(arrayType);
    wasmModule->AppendField(std::move(field));
    return Result::Ok;
}

Result BinaryReaderIR::OnImportCount(Index count) {
    WABT_TRY
    wasmModule->imports.reserve(count);
    WABT_CATCH_BAD_ALLOC
    return Result::Ok;
}

Result BinaryReaderIR::OnImportFunc(Index importIndex, StringRef moduleName,
                                    StringRef fieldName, Index funcIndex,
                                    Index sigIndex) {
    auto import = std::make_unique<FuncImport>();
    import->module_name = moduleName.str();
    import->field_name = fieldName.str();
    setFuncDeclaration(&import->func.decl, Var(sigIndex));
    wasmModule->AppendField(
        std::make_unique<ImportModuleField>(std::move(import)));
    return Result::Ok;
}

Result BinaryReaderIR::OnImportTable(Index importIndex, StringRef moduleName,
                                     StringRef fieldName, Index tableIndex,
                                     Type elemType, const Limits *elemLimits) {
    auto import = std::make_unique<TableImport>();
    import->module_name = moduleName.str();
    import->field_name = fieldName.str();
    import->table.elem_limits = *elemLimits;
    import->table.elem_type = elemType;
    wasmModule->AppendField(
        std::make_unique<ImportModuleField>(std::move(import)));
    return Result::Ok;
}

Result BinaryReaderIR::OnImportMemory(Index importIndex, StringRef moduleName,
                                      StringRef fieldName, Index memoryIndex,
                                      const Limits *pageLimits) {
    auto import = std::make_unique<MemoryImport>();
    import->module_name = moduleName.str();
    import->field_name = fieldName.str();
    import->memory.page_limits = *pageLimits;
    wasmModule->AppendField(
        std::make_unique<ImportModuleField>(std::move(import)));
    return Result::Ok;
}

Result BinaryReaderIR::OnImportGlobal(Index importIndex, StringRef moduleName,
                                      StringRef fieldName, Index globalIndex,
                                      Type type, bool isMutable) {
    auto import = std::make_unique<GlobalImport>();
    import->module_name = moduleName.str();
    import->field_name = fieldName.str();
    import->global.type = type;
    import->global.mutable_ = isMutable;
    wasmModule->AppendField(
        std::make_unique<ImportModuleField>(std::move(import)));
    wasmModule->features_used.simd |= (type == Type::V128);
    return Result::Ok;
}

Result BinaryReaderIR::OnImportTag(Index importIndex, StringRef moduleName,
                                   StringRef fieldName, Index tagIndex,
                                   Index sigIndex) {
    auto import = std::make_unique<TagImport>();
    import->module_name = moduleName.str();
    import->field_name = fieldName.str();
    setFuncDeclaration(&import->tag.decl, Var(sigIndex));
    wasmModule->AppendField(
        std::make_unique<ImportModuleField>(std::move(import)));
    wasmModule->features_used.exceptions = true;
    return Result::Ok;
}

Result BinaryReaderIR::OnFunctionCount(Index count) {
    WABT_TRY
    wasmModule->funcs.reserve(wasmModule->num_func_imports + count);
    WABT_CATCH_BAD_ALLOC
    return Result::Ok;
}

Result BinaryReaderIR::OnFunction(Index index, Index sigIndex) {
    auto field = std::make_unique<FuncModuleField>();
    Func &func = field->func;
    setFuncDeclaration(&func.decl, Var(sigIndex));
    wasmModule->AppendField(std::move(field));
    return Result::Ok;
}

Result BinaryReaderIR::OnTableCount(Index count) {
    WABT_TRY
    wasmModule->tables.reserve(wasmModule->num_table_imports + count);
    WABT_CATCH_BAD_ALLOC
    return Result::Ok;
}

Result BinaryReaderIR::OnTable(Index index, Type elemType,
                               const Limits *elemLimits) {
    auto field = std::make_unique<TableModuleField>();
    Table &table = field->table;
    table.elem_limits = *elemLimits;
    table.elem_type = elemType;
    wasmModule->AppendField(std::move(field));
    return Result::Ok;
}

Result BinaryReaderIR::OnMemoryCount(Index count) {
    WABT_TRY
    wasmModule->memories.reserve(wasmModule->num_memory_imports + count);
    WABT_CATCH_BAD_ALLOC
    return Result::Ok;
}

Result BinaryReaderIR::OnMemory(Index index, const Limits *pageLimits) {
    auto field = std::make_unique<MemoryModuleField>();
    Memory &memory = field->memory;
    memory.page_limits = *pageLimits;
    wasmModule->AppendField(std::move(field));
    return Result::Ok;
}

Result BinaryReaderIR::OnGlobalCount(Index count) {
    WABT_TRY
    wasmModule->globals.reserve(wasmModule->num_global_imports + count);
    WABT_CATCH_BAD_ALLOC
    return Result::Ok;
}

Result BinaryReaderIR::BeginGlobal(Index index, Type type, bool isMutable) {
    auto field = std::make_unique<GlobalModuleField>();
    Global &global = field->global;
    global.type = type;
    global.mutable_ = isMutable;
    wasmModule->AppendField(std::move(field));
    wasmModule->features_used.simd |= (type == Type::V128);
    return Result::Ok;
}

Result BinaryReaderIR::BeginGlobalInitExpr(Index index) {
    assert(index == wasmModule->globals.size() - 1);
    Global *global = wasmModule->globals[index];
    return beginInitExpr(&global->init_expr);
}

Result BinaryReaderIR::EndGlobalInitExpr(Index index) {
    return endInitExpr();
}

Result BinaryReaderIR::OnExportCount(Index count) {
    WABT_TRY
    wasmModule->exports.reserve(count);
    WABT_CATCH_BAD_ALLOC
    return Result::Ok;
}

Result BinaryReaderIR::OnExport(Index index, ExternalKind kind, Index itemIndex,
                                StringRef name) {
    auto field = std::make_unique<ExportModuleField>();
    Export &exportObj = field->export_;
    exportObj.name = name.str();
    exportObj.var = Var(itemIndex);
    exportObj.kind = kind;
    wasmModule->AppendField(std::move(field));
    return Result::Ok;
}

Result BinaryReaderIR::OnStartFunction(Index funcIndex) {
    Var start(funcIndex);
    wasmModule->AppendField(std::make_unique<StartModuleField>(start));
    return Result::Ok;
}

Result BinaryReaderIR::OnFunctionBodyCount(Index count) {
    wasmModule->code_count_size = state->offset - codeStart;

    // Can hit this case on a malformed module if we don't stop on first error.
    if (wasmModule->num_func_imports + count != wasmModule->funcs.size()) {
        printError("number of imported func + func count in code section does "
                   "not match "
                   "actual number of funcs in module");
        return Result::Error;
    }
    return Result::Ok;
}

Result BinaryReaderIR::BeginFunctionBody(Index index, Offset size) {
    currentFunc = wasmModule->funcs[index];
    return pushLabel(LabelType::Func, &currentFunc->exprs);
}

Result BinaryReaderIR::OnLocalDecl(Index declIndex, Index count, Type type) {
    currentFunc->local_types.AppendDecl(type, count);

    if (currentFunc->GetNumLocals() > kMaxFunctionLocals) {
        printError("function local count exceeds maximum value");
        return Result::Error;
    }

    wasmModule->features_used.simd |= (type == Type::V128);
    return Result::Ok;
}

Result BinaryReaderIR::OnOpcode(Opcode opcode) {
    std::unique_ptr<CodeMetadataExpr> metadata =
        codeMetadataQueue.popMatch(currentFunc, state->offset - 1);
    if (metadata) {
        return appendExpr(std::move(metadata));
    }
    wasmModule->features_used.simd |= (opcode.GetResultType() == Type::V128);
    wasmModule->features_used.threads |= (opcode.GetPrefix() == 0xfe);
    return Result::Ok;
}

Result BinaryReaderIR::OnAtomicLoadExpr(Opcode opcode, Index memidx,
                                        Address alignmentLog2, Address offset) {
    return appendExpr(std::make_unique<AtomicLoadExpr>(
        opcode, Var(memidx), 1 << alignmentLog2, offset));
}

Result BinaryReaderIR::OnAtomicStoreExpr(Opcode opcode, Index memidx,
                                         Address alignmentLog2,
                                         Address offset) {
    return appendExpr(std::make_unique<AtomicStoreExpr>(
        opcode, Var(memidx), 1 << alignmentLog2, offset));
}

Result BinaryReaderIR::OnAtomicRmwExpr(Opcode opcode, Index memidx,
                                       Address alignmentLog2, Address offset) {
    return appendExpr(std::make_unique<AtomicRmwExpr>(
        opcode, Var(memidx), 1 << alignmentLog2, offset));
}

Result BinaryReaderIR::OnAtomicRmwCmpxchgExpr(Opcode opcode, Index memidx,
                                              Address alignmentLog2,
                                              Address offset) {
    return appendExpr(std::make_unique<AtomicRmwCmpxchgExpr>(
        opcode, Var(memidx), 1 << alignmentLog2, offset));
}

Result BinaryReaderIR::OnAtomicWaitExpr(Opcode opcode, Index memidx,
                                        Address alignmentLog2, Address offset) {
    return appendExpr(std::make_unique<AtomicWaitExpr>(
        opcode, Var(memidx), 1 << alignmentLog2, offset));
}

Result BinaryReaderIR::OnAtomicFenceExpr(uint32_t consistencyModel) {
    return appendExpr(std::make_unique<AtomicFenceExpr>(consistencyModel));
}

Result BinaryReaderIR::OnAtomicNotifyExpr(Opcode opcode, Index memidx,
                                          Address alignmentLog2,
                                          Address offset) {
    return appendExpr(std::make_unique<AtomicNotifyExpr>(
        opcode, Var(memidx), 1 << alignmentLog2, offset));
}

Result BinaryReaderIR::OnBinaryExpr(Opcode opcode) {
    return appendExpr(std::make_unique<BinaryExpr>(opcode));
}

Result BinaryReaderIR::OnBlockExpr(Type sigType) {
    auto expr = std::make_unique<BlockExpr>();
    setBlockDeclaration(&expr->block.decl, sigType);
    ExprList *exprList = &expr->block.exprs;
    CHECK_RESULT(appendExpr(std::move(expr)));
    return pushLabel(LabelType::Block, exprList);
}

Result BinaryReaderIR::OnBrExpr(Index depth) {
    return appendExpr(std::make_unique<BrExpr>(Var(depth)));
}

Result BinaryReaderIR::OnBrIfExpr(Index depth) {
    return appendExpr(std::make_unique<BrIfExpr>(Var(depth)));
}

Result BinaryReaderIR::OnBrTableExpr(Index numTargets, Index *targetDepths,
                                     Index defaultTargetDepth) {
    auto expr = std::make_unique<BrTableExpr>();
    expr->default_target = Var(defaultTargetDepth);
    expr->targets.resize(numTargets);
    for (Index i = 0; i < numTargets; ++i) {
        expr->targets[i] = Var(targetDepths[i]);
    }
    return appendExpr(std::move(expr));
}

Result BinaryReaderIR::OnCallExpr(Index funcIndex) {
    return appendExpr(std::make_unique<CallExpr>(Var(funcIndex)));
}

Result BinaryReaderIR::OnCallIndirectExpr(Index sigIndex, Index tableIndex) {
    auto expr = std::make_unique<CallIndirectExpr>();
    setFuncDeclaration(&expr->decl, Var(sigIndex));
    expr->table = Var(tableIndex);
    return appendExpr(std::move(expr));
}

Result BinaryReaderIR::OnCallRefExpr() {
    return appendExpr(std::make_unique<CallRefExpr>());
}

Result BinaryReaderIR::OnReturnCallExpr(Index funcIndex) {
    if (currentFunc) {
        // syntactically, a return_call expr can occur in an init expression
        // (outside a function)
        currentFunc->features_used.tailcall = true;
    }
    return appendExpr(std::make_unique<ReturnCallExpr>(Var(funcIndex)));
}

Result BinaryReaderIR::OnReturnCallIndirectExpr(Index sigIndex,
                                                Index tableIndex) {
    if (currentFunc) {
        // syntactically, a return_call_indirect expr can occur in an init
        // expression (outside a function)
        currentFunc->features_used.tailcall = true;
    }
    auto expr = std::make_unique<ReturnCallIndirectExpr>();
    setFuncDeclaration(&expr->decl, Var(sigIndex));
    expr->table = Var(tableIndex);
    FuncType *type = wasmModule->GetFuncType(Var(sigIndex));
    if (type) {
        type->features_used.tailcall = true;
    }
    return appendExpr(std::move(expr));
}

Result BinaryReaderIR::OnCompareExpr(Opcode opcode) {
    return appendExpr(std::make_unique<CompareExpr>(opcode));
}

Result BinaryReaderIR::OnConvertExpr(Opcode opcode) {
    return appendExpr(std::make_unique<ConvertExpr>(opcode));
}

Result BinaryReaderIR::OnDropExpr() {
    return appendExpr(std::make_unique<DropExpr>());
}

Result BinaryReaderIR::OnElseExpr() {
    LabelNode *label;
    Expr *expr;
    CHECK_RESULT(topLabelExpr(&label, &expr));

    if (label->labelType == LabelType::If) {
        auto *ifExpr = cast<IfExpr>(expr);
        label->exprs = &ifExpr->false_;
        label->labelType = LabelType::Else;
    } else {
        printError("else expression without matching if");
        return Result::Error;
    }

    return Result::Ok;
}

Result BinaryReaderIR::OnEndExpr() {
    if (labelStack.size() > 1) {
        LabelNode *label;
        Expr *expr;
        CHECK_RESULT(topLabelExpr(&label, &expr));
    }

    return popLabel();
}

Result BinaryReaderIR::OnF32ConstExpr(uint32_t valueBits) {
    return appendExpr(std::make_unique<ConstExpr>(Const::F32(valueBits)));
}

Result BinaryReaderIR::OnF64ConstExpr(uint64_t valueBits) {
    return appendExpr(std::make_unique<ConstExpr>(Const::F64(valueBits)));
}

Result BinaryReaderIR::OnV128ConstExpr(v128 valueBits) {
    return appendExpr(std::make_unique<ConstExpr>(Const::V128(valueBits)));
}

Result BinaryReaderIR::OnGlobalGetExpr(Index globalIndex) {
    return appendExpr(std::make_unique<GlobalGetExpr>(Var(globalIndex)));
}

Result BinaryReaderIR::OnLocalGetExpr(Index localIndex) {
    return appendExpr(std::make_unique<LocalGetExpr>(Var(localIndex)));
}

Result BinaryReaderIR::OnI32ConstExpr(uint32_t value) {
    return appendExpr(std::make_unique<ConstExpr>(Const::I32(value)));
}

Result BinaryReaderIR::OnI64ConstExpr(uint64_t value) {
    return appendExpr(std::make_unique<ConstExpr>(Const::I64(value)));
}

Result BinaryReaderIR::OnIfExpr(Type sigType) {
    auto expr = std::make_unique<IfExpr>();
    setBlockDeclaration(&expr->true_.decl, sigType);
    ExprList *exprList = &expr->true_.exprs;
    CHECK_RESULT(appendExpr(std::move(expr)));
    return pushLabel(LabelType::If, exprList);
}

Result BinaryReaderIR::OnLoadExpr(Opcode opcode, Index memidx,
                                  Address alignmentLog2, Address offset) {
    return appendExpr(std::make_unique<LoadExpr>(opcode, Var(memidx),
                                                 1 << alignmentLog2, offset));
}

Result BinaryReaderIR::OnLoopExpr(Type sigType) {
    auto expr = std::make_unique<LoopExpr>();
    setBlockDeclaration(&expr->block.decl, sigType);
    ExprList *exprList = &expr->block.exprs;
    CHECK_RESULT(appendExpr(std::move(expr)));
    return pushLabel(LabelType::Loop, exprList);
}

Result BinaryReaderIR::OnMemoryCopyExpr(Index destmemidx, Index srcmemidx) {
    return appendExpr(
        std::make_unique<MemoryCopyExpr>(Var(destmemidx), Var(srcmemidx)));
}

Result BinaryReaderIR::OnDataDropExpr(Index segment) {
    return appendExpr(std::make_unique<DataDropExpr>(Var(segment)));
}

Result BinaryReaderIR::OnMemoryFillExpr(Index memidx) {
    return appendExpr(std::make_unique<MemoryFillExpr>(Var(memidx)));
}

Result BinaryReaderIR::OnMemoryGrowExpr(Index memidx) {
    return appendExpr(std::make_unique<MemoryGrowExpr>(Var(memidx)));
}

Result BinaryReaderIR::OnMemoryInitExpr(Index segment, Index memidx) {
    return appendExpr(
        std::make_unique<MemoryInitExpr>(Var(segment), Var(memidx)));
}

Result BinaryReaderIR::OnMemorySizeExpr(Index memidx) {
    return appendExpr(std::make_unique<MemorySizeExpr>(Var(memidx)));
}

Result BinaryReaderIR::OnTableCopyExpr(Index dstIndex, Index srcIndex) {
    return appendExpr(
        std::make_unique<TableCopyExpr>(Var(dstIndex), Var(srcIndex)));
}

Result BinaryReaderIR::OnElemDropExpr(Index segment) {
    return appendExpr(std::make_unique<ElemDropExpr>(Var(segment)));
}

Result BinaryReaderIR::OnTableInitExpr(Index segment, Index tableIndex) {
    return appendExpr(
        std::make_unique<TableInitExpr>(Var(segment), Var(tableIndex)));
}

Result BinaryReaderIR::OnTableGetExpr(Index tableIndex) {
    return appendExpr(std::make_unique<TableGetExpr>(Var(tableIndex)));
}

Result BinaryReaderIR::OnTableSetExpr(Index tableIndex) {
    return appendExpr(std::make_unique<TableSetExpr>(Var(tableIndex)));
}

Result BinaryReaderIR::OnTableGrowExpr(Index tableIndex) {
    return appendExpr(std::make_unique<TableGrowExpr>(Var(tableIndex)));
}

Result BinaryReaderIR::OnTableSizeExpr(Index tableIndex) {
    return appendExpr(std::make_unique<TableSizeExpr>(Var(tableIndex)));
}

Result BinaryReaderIR::OnTableFillExpr(Index tableIndex) {
    return appendExpr(std::make_unique<TableFillExpr>(Var(tableIndex)));
}

Result BinaryReaderIR::OnRefFuncExpr(Index funcIndex) {
    return appendExpr(std::make_unique<RefFuncExpr>(Var(funcIndex)));
}

Result BinaryReaderIR::OnRefNullExpr(Type type) {
    return appendExpr(std::make_unique<RefNullExpr>(type));
}

Result BinaryReaderIR::OnRefIsNullExpr() {
    return appendExpr(std::make_unique<RefIsNullExpr>());
}

Result BinaryReaderIR::OnNopExpr() {
    return appendExpr(std::make_unique<NopExpr>());
}

Result BinaryReaderIR::OnRethrowExpr(Index depth) {
    return appendExpr(std::make_unique<RethrowExpr>(Var(depth)));
}

Result BinaryReaderIR::OnReturnExpr() {
    return appendExpr(std::make_unique<ReturnExpr>());
}

Result BinaryReaderIR::OnSelectExpr(Index resultCount, Type *resultTypes) {
    TypeVector results;
    results.assign(resultTypes, resultTypes + resultCount);
    return appendExpr(std::make_unique<SelectExpr>(results));
}

Result BinaryReaderIR::OnGlobalSetExpr(Index globalIndex) {
    return appendExpr(std::make_unique<GlobalSetExpr>(Var(globalIndex)));
}

Result BinaryReaderIR::OnLocalSetExpr(Index localIndex) {
    return appendExpr(std::make_unique<LocalSetExpr>(Var(localIndex)));
}

Result BinaryReaderIR::OnStoreExpr(Opcode opcode, Index memidx,
                                   Address alignmentLog2, Address offset) {
    return appendExpr(std::make_unique<StoreExpr>(opcode, Var(memidx),
                                                  1 << alignmentLog2, offset));
}

Result BinaryReaderIR::OnThrowExpr(Index tagIndex) {
    return appendExpr(std::make_unique<ThrowExpr>(Var(tagIndex)));
}

Result BinaryReaderIR::OnLocalTeeExpr(Index localIndex) {
    return appendExpr(std::make_unique<LocalTeeExpr>(Var(localIndex)));
}

Result BinaryReaderIR::OnTryExpr(Type sigType) {
    auto exprPtr = std::make_unique<TryExpr>();
    // Save expr so it can be used below, after expr_ptr has been moved.
    TryExpr *expr = exprPtr.get();
    ExprList *exprList = &expr->block.exprs;
    setBlockDeclaration(&expr->block.decl, sigType);
    CHECK_RESULT(appendExpr(std::move(exprPtr)));
    wasmModule->features_used.exceptions = true;
    return pushLabel(LabelType::Try, exprList, expr);
}

Result BinaryReaderIR::appendCatch(Catch &&catchObj) {
    LabelNode *label = nullptr;
    CHECK_RESULT(topLabel(&label));

    if (label->labelType != LabelType::Try) {
        printError("catch not inside try block");
        return Result::Error;
    }

    auto *tryObj = cast<TryExpr>(label->context);

    if (catchObj.IsCatchAll() && !tryObj->catches.empty() &&
        tryObj->catches.back().IsCatchAll()) {
        printError("only one catch_all allowed in try block");
        return Result::Error;
    }

    if (tryObj->kind == TryKind::Plain) {
        tryObj->kind = TryKind::Catch;
    } else if (tryObj->kind != TryKind::Catch) {
        printError("catch not allowed in try-delegate");
        return Result::Error;
    }

    tryObj->catches.push_back(std::move(catchObj));
    label->exprs = &tryObj->catches.back().exprs;
    return Result::Ok;
}

Result BinaryReaderIR::OnCatchExpr(Index exceptIndex) {
    return appendCatch(Catch(Var(exceptIndex)));
}

Result BinaryReaderIR::OnCatchAllExpr() {
    return appendCatch(Catch());
}

Result BinaryReaderIR::OnDelegateExpr(Index depth) {
    LabelNode *label = nullptr;
    CHECK_RESULT(topLabel(&label));

    if (label->labelType != LabelType::Try) {
        printError("delegate not inside try block");
        return Result::Error;
    }

    auto *tryObj = cast<TryExpr>(label->context);

    if (tryObj->kind == TryKind::Plain) {
        tryObj->kind = TryKind::Delegate;
    } else if (tryObj->kind != TryKind::Delegate) {
        printError("delegate not allowed in try-catch");
        return Result::Error;
    }

    tryObj->delegate_target = Var(depth);

    popLabel();
    return Result::Ok;
}

Result BinaryReaderIR::OnUnaryExpr(Opcode opcode) {
    return appendExpr(std::make_unique<UnaryExpr>(opcode));
}

Result BinaryReaderIR::OnTernaryExpr(Opcode opcode) {
    return appendExpr(std::make_unique<TernaryExpr>(opcode));
}

Result BinaryReaderIR::OnUnreachableExpr() {
    return appendExpr(std::make_unique<UnreachableExpr>());
}

Result BinaryReaderIR::EndFunctionBody(Index index) {
    currentFunc = nullptr;
    if (!labelStack.empty()) {
        labelStack.clear();
    }
    return Result::Ok;
}

Result BinaryReaderIR::OnSimdLaneOpExpr(Opcode opcode, uint64_t value) {
    return appendExpr(std::make_unique<SimdLaneOpExpr>(opcode, value));
}

Result BinaryReaderIR::OnSimdLoadLaneExpr(Opcode opcode, Index memidx,
                                          Address alignmentLog2, Address offset,
                                          uint64_t value) {
    return appendExpr(std::make_unique<SimdLoadLaneExpr>(
        opcode, Var(memidx), 1 << alignmentLog2, offset, value));
}

Result BinaryReaderIR::OnSimdStoreLaneExpr(Opcode opcode, Index memidx,
                                           Address alignmentLog2,
                                           Address offset, uint64_t value) {
    return appendExpr(std::make_unique<SimdStoreLaneExpr>(
        opcode, Var(memidx), 1 << alignmentLog2, offset, value));
}

Result BinaryReaderIR::OnSimdShuffleOpExpr(Opcode opcode, v128 value) {
    return appendExpr(std::make_unique<SimdShuffleOpExpr>(opcode, value));
}

Result BinaryReaderIR::OnLoadSplatExpr(Opcode opcode, Index memidx,
                                       Address alignmentLog2, Address offset) {
    return appendExpr(std::make_unique<LoadSplatExpr>(
        opcode, Var(memidx), 1 << alignmentLog2, offset));
}

Result BinaryReaderIR::OnLoadZeroExpr(Opcode opcode, Index memidx,
                                      Address alignmentLog2, Address offset) {
    return appendExpr(std::make_unique<LoadZeroExpr>(
        opcode, Var(memidx), 1 << alignmentLog2, offset));
}

Result BinaryReaderIR::OnElemSegmentCount(Index count) {
    WABT_TRY
    wasmModule->elem_segments.reserve(count);
    WABT_CATCH_BAD_ALLOC
    return Result::Ok;
}

Result BinaryReaderIR::BeginElemSegment(Index index, Index tableIndex,
                                        uint8_t flags) {
    auto field = std::make_unique<ElemSegmentModuleField>();
    ElemSegment &elemSegment = field->elem_segment;
    elemSegment.table_var = Var(tableIndex);
    if ((flags & SegDeclared) == SegDeclared) {
        elemSegment.kind = SegmentKind::Declared;
    } else if ((flags & SegPassive) == SegPassive) {
        elemSegment.kind = SegmentKind::Passive;
    } else {
        elemSegment.kind = SegmentKind::Active;
    }
    wasmModule->AppendField(std::move(field));
    return Result::Ok;
}

Result BinaryReaderIR::beginInitExpr(ExprList *expr) {
    return pushLabel(LabelType::InitExpr, expr);
}

Result BinaryReaderIR::BeginElemSegmentInitExpr(Index index) {
    assert(index == wasmModule->elem_segments.size() - 1);
    ElemSegment *segment = wasmModule->elem_segments[index];
    return beginInitExpr(&segment->offset);
}

Result BinaryReaderIR::endInitExpr() {
    if (!labelStack.empty()) {
        printError("init expression missing end marker");
        return Result::Error;
    }
    return Result::Ok;
}

Result BinaryReaderIR::EndElemSegmentInitExpr(Index index) {
    return endInitExpr();
}

Result BinaryReaderIR::OnElemSegmentElemType(Index index, Type elemType) {
    assert(index == wasmModule->elem_segments.size() - 1);
    ElemSegment *segment = wasmModule->elem_segments[index];
    segment->elem_type = elemType;
    return Result::Ok;
}

Result BinaryReaderIR::OnElemSegmentElemExprCount(Index index, Index count) {
    assert(index == wasmModule->elem_segments.size() - 1);
    ElemSegment *segment = wasmModule->elem_segments[index];
    WABT_TRY
    segment->elem_exprs.reserve(count);
    WABT_CATCH_BAD_ALLOC
    return Result::Ok;
}

Result BinaryReaderIR::BeginElemExpr(Index elemIndex, Index exprIndex) {
    assert(elemIndex == wasmModule->elem_segments.size() - 1);
    ElemSegment *segment = wasmModule->elem_segments[elemIndex];
    assert(exprIndex == segment->elem_exprs.size());
    segment->elem_exprs.emplace_back();
    return beginInitExpr(&segment->elem_exprs.back());
}

Result BinaryReaderIR::EndElemExpr(Index elemIndex, Index exprIndex) {
    return endInitExpr();
}

Result BinaryReaderIR::OnDataSegmentCount(Index count) {
    WABT_TRY
    wasmModule->data_segments.reserve(count);
    WABT_CATCH_BAD_ALLOC
    return Result::Ok;
}

Result BinaryReaderIR::BeginDataSegment(Index index, Index memoryIndex,
                                        uint8_t flags) {
    auto field = std::make_unique<DataSegmentModuleField>();
    DataSegment &dataSegment = field->data_segment;
    dataSegment.memory_var = Var(memoryIndex);
    if ((flags & SegPassive) == SegPassive) {
        dataSegment.kind = SegmentKind::Passive;
    } else {
        dataSegment.kind = SegmentKind::Active;
    }
    wasmModule->AppendField(std::move(field));
    return Result::Ok;
}

Result BinaryReaderIR::BeginDataSegmentInitExpr(Index index) {
    assert(index == wasmModule->data_segments.size() - 1);
    DataSegment *segment = wasmModule->data_segments[index];
    return beginInitExpr(&segment->offset);
}

Result BinaryReaderIR::EndDataSegmentInitExpr(Index index) {
    return endInitExpr();
}

Result BinaryReaderIR::OnDataSegmentData(Index index, const void *data,
                                         Address size) {
    wasmModule->data_segment_offsets[index] = state->offset - size - dataStart;
    assert(index == wasmModule->data_segments.size() - 1);
    DataSegment *segment = wasmModule->data_segments[index];
    segment->data.resize(size);
    if (size > 0) {
        memcpy(segment->data.data(), data, size);
    }
    return Result::Ok;
}

Result BinaryReaderIR::OnFunctionNamesCount(Index count) {
    if (count > wasmModule->funcs.size()) {
        return Result::Error;
    }
    return Result::Ok;
}

static std::string makeDollarName(StringRef name) {
    return std::string("$") + name.str();
}

Result BinaryReaderIR::OnModuleName(StringRef name) {
    if (name.empty()) {
        return Result::Ok;
    }

    wasmModule->name = makeDollarName(name);
    return Result::Ok;
}

Result BinaryReaderIR::setGlobalName(Index index, StringRef name) {
    if (name.empty()) {
        return Result::Ok;
    }
    if (index >= wasmModule->globals.size()) {
        printError("invalid global index: %" PRIindex, index);
        return Result::Error;
    }
    Global *glob = wasmModule->globals[index];
    std::string dollarName =
        getUniqueName(&wasmModule->global_bindings, makeDollarName(name));
    glob->name = dollarName;
    wasmModule->global_bindings.emplace(dollarName, Binding(index));
    return Result::Ok;
}

Result BinaryReaderIR::setFunctionName(Index index, StringRef name) {
    if (name.empty()) {
        return Result::Ok;
    }
    if (index >= wasmModule->funcs.size()) {
        printError("invalid function index: %" PRIindex, index);
        return Result::Error;
    }
    Func *func = wasmModule->funcs[index];
    std::string dollarName =
        getUniqueName(&wasmModule->func_bindings, makeDollarName(name));
    func->name = dollarName;
    wasmModule->func_bindings.emplace(dollarName, Binding(index));
    return Result::Ok;
}

Result BinaryReaderIR::setTypeName(Index index, StringRef name) {
    if (name.empty()) {
        return Result::Ok;
    }
    if (index >= wasmModule->types.size()) {
        printError("invalid type index: %" PRIindex, index);
        return Result::Error;
    }
    TypeEntry *type = wasmModule->types[index];
    std::string dollarName =
        getUniqueName(&wasmModule->type_bindings, makeDollarName(name));
    type->name = dollarName;
    wasmModule->type_bindings.emplace(dollarName, Binding(index));
    return Result::Ok;
}

Result BinaryReaderIR::setTableName(Index index, StringRef name) {
    if (name.empty()) {
        return Result::Ok;
    }
    if (index >= wasmModule->tables.size()) {
        printError("invalid table index: %" PRIindex, index);
        return Result::Error;
    }
    Table *table = wasmModule->tables[index];
    std::string dollarName =
        getUniqueName(&wasmModule->table_bindings, makeDollarName(name));
    table->name = dollarName;
    wasmModule->table_bindings.emplace(dollarName, Binding(index));
    return Result::Ok;
}

Result BinaryReaderIR::setDataSegmentName(Index index, StringRef name) {
    if (name.empty()) {
        return Result::Ok;
    }
    if (index >= wasmModule->data_segments.size()) {
        printError("invalid data segment index: %" PRIindex, index);
        return Result::Error;
    }
    DataSegment *segment = wasmModule->data_segments[index];
    std::string dollarName =
        getUniqueName(&wasmModule->data_segment_bindings, makeDollarName(name));
    segment->name = dollarName;
    wasmModule->data_segment_bindings.emplace(dollarName, Binding(index));
    return Result::Ok;
}

Result BinaryReaderIR::setElemSegmentName(Index index, StringRef name) {
    if (name.empty()) {
        return Result::Ok;
    }
    if (index >= wasmModule->elem_segments.size()) {
        printError("invalid elem segment index: %" PRIindex, index);
        return Result::Error;
    }
    ElemSegment *segment = wasmModule->elem_segments[index];
    std::string dollarName =
        getUniqueName(&wasmModule->elem_segment_bindings, makeDollarName(name));
    segment->name = dollarName;
    wasmModule->elem_segment_bindings.emplace(dollarName, Binding(index));
    return Result::Ok;
}

Result BinaryReaderIR::setMemoryName(Index index, StringRef name) {
    if (name.empty()) {
        return Result::Ok;
    }
    if (index >= wasmModule->memories.size()) {
        printError("invalid memory index: %" PRIindex, index);
        return Result::Error;
    }
    Memory *memory = wasmModule->memories[index];
    std::string dollarName =
        getUniqueName(&wasmModule->memory_bindings, makeDollarName(name));
    memory->name = dollarName;
    wasmModule->memory_bindings.emplace(dollarName, Binding(index));
    return Result::Ok;
}

Result BinaryReaderIR::setTagName(Index index, StringRef name) {
    if (name.empty()) {
        return Result::Ok;
    }
    if (index >= wasmModule->tags.size()) {
        printError("invalid tag index: %" PRIindex, index);
        return Result::Error;
    }
    Tag *tag = wasmModule->tags[index];
    std::string dollarName =
        getUniqueName(&wasmModule->tag_bindings, makeDollarName(name));
    tag->name = dollarName;
    wasmModule->tag_bindings.emplace(dollarName, Binding(index));
    return Result::Ok;
}

Result BinaryReaderIR::OnFunctionName(Index index, StringRef name) {
    return setFunctionName(index, name);
}

Result BinaryReaderIR::OnNameEntry(NameSectionSubsection type, Index index,
                                   StringRef name) {
    switch (type) {
    // TODO(sbc): remove OnFunctionName in favor of just using
    // OnNameEntry so that this works
    case NameSectionSubsection::Function:
    case NameSectionSubsection::Local:
    case NameSectionSubsection::Module:
    case NameSectionSubsection::Label:
        break;
    case NameSectionSubsection::Type:
        setTypeName(index, name);
        break;
    case NameSectionSubsection::Tag:
        setTagName(index, name);
        break;
    case NameSectionSubsection::Global:
        setGlobalName(index, name);
        break;
    case NameSectionSubsection::Table:
        setTableName(index, name);
        break;
    case NameSectionSubsection::DataSegment:
        setDataSegmentName(index, name);
        break;
    case NameSectionSubsection::Memory:
        setMemoryName(index, name);
        break;
    case NameSectionSubsection::ElemSegment:
        setElemSegmentName(index, name);
        break;
    }
    return Result::Ok;
}

Result BinaryReaderIR::OnLocalNameLocalCount(Index index, Index count) {
    assert(index < wasmModule->funcs.size());
    Func *func = wasmModule->funcs[index];
    Index numParamsAndLocals = func->GetNumParamsAndLocals();
    if (count > numParamsAndLocals) {
        printError("expected local name count (%" PRIindex
                   ") <= local count (%" PRIindex ")",
                   count, numParamsAndLocals);
        return Result::Error;
    }
    return Result::Ok;
}

Result BinaryReaderIR::BeginCodeMetadataSection(StringRef name, Offset size) {
    currentMetadataName = name;
    return Result::Ok;
}

Result BinaryReaderIR::OnCodeMetadataFuncCount(Index count) {
    return Result::Ok;
}

Result BinaryReaderIR::OnCodeMetadataCount(Index functionIndex, Index count) {
    codeMetadataQueue.pushFunc(wasmModule->funcs[functionIndex]);
    return Result::Ok;
}

Result BinaryReaderIR::OnCodeMetadata(Offset offset, const void *data,
                                      Address size) {
    std::vector<uint8_t> dataVector(static_cast<const uint8_t *>(data),
                                    static_cast<const uint8_t *>(data) + size);
    auto meta = std::make_unique<CodeMetadataExpr>(currentMetadataName,
                                                   std::move(dataVector));
    codeMetadataQueue.pushMetadata(std::move(meta));
    return Result::Ok;
}

Result BinaryReaderIR::OnLocalName(Index funcIndex, Index localIndex,
                                   StringRef name) {
    if (name.empty()) {
        return Result::Ok;
    }

    Func *func = wasmModule->funcs[funcIndex];
    func->bindings.emplace(getUniqueName(&func->bindings, makeDollarName(name)),
                           Binding(localIndex));
    return Result::Ok;
}

Result BinaryReaderIR::OnTagType(Index index, Index sigIndex) {
    auto field = std::make_unique<TagModuleField>();
    Tag &tag = field->tag;
    setFuncDeclaration(&tag.decl, Var(sigIndex));
    wasmModule->AppendField(std::move(field));
    wasmModule->features_used.exceptions = true;
    return Result::Ok;
}

Result BinaryReaderIR::OnDataSymbol(Index index, uint32_t flags, StringRef name,
                                    Index segment, uint32_t offset,
                                    uint32_t size) {
    // assert(offset == 0);
    assert(index == wasmModule->symbols.size());
    SymbolBinding binding =
        static_cast<SymbolBinding>(flags & WABT_SYMBOL_MASK_BINDING);
    wasmModule->symbols.push_back(std::make_unique<WASMModule::DataSymbol>(
        name.str(), wasmModule, segment, offset,
        !(flags & WABT_SYMBOL_FLAG_UNDEFINED), binding == SymbolBinding::Weak,
        binding == SymbolBinding::Local));
    if (!(flags & WABT_SYMBOL_FLAG_UNDEFINED))
        wasmModule->data_symbol_map[index] = segment;
    if (!name.empty())
        wasmModule->data_symbol_name_map[index] = name.str();
    if (name.empty()) {
        return Result::Ok;
    }
    if (flags & WABT_SYMBOL_FLAG_UNDEFINED) {
        // Refers to data in another file, `segment` not valid.
        return Result::Ok;
    }
    if (offset) {
        // If it is pointing into the data segment, then it's not really naming
        // the whole segment.
        return Result::Ok;
    }
    if (segment >= wasmModule->data_segments.size()) {
        printError("invalid data segment index: %" PRIindex, segment);
        return Result::Error;
    }
    DataSegment *seg = wasmModule->data_segments[segment];
    std::string dollarName =
        getUniqueName(&wasmModule->data_segment_bindings, makeDollarName(name));
    seg->name = dollarName;
    wasmModule->data_segment_bindings.emplace(dollarName, Binding(segment));
    return Result::Ok;
}

Result BinaryReaderIR::OnFunctionSymbol(Index index, uint32_t flags,
                                        StringRef name, Index funcIndex) {
    wasmModule->function_symbol_map[index] = funcIndex;
    {
        StringRef symbolName = name;
        if (!(flags & WABT_SYMBOL_FLAG_EXPLICIT_NAME)) {
            Index funcImportIndex = 0;
            for (auto &import : wasmModule->imports) {
                if (import->kind() != ExternalKind::Func)
                    continue;
                if (funcImportIndex == funcIndex) {
                    symbolName = import->field_name;
                    break;
                }
                funcImportIndex++;
            }
        }
        assert(index == wasmModule->symbols.size());
        SymbolBinding binding =
            static_cast<SymbolBinding>(flags & WABT_SYMBOL_MASK_BINDING);
        wasmModule->symbols.push_back(std::make_unique<WASMModule::FuncSymbol>(
            symbolName.str(), wasmModule, funcIndex,
            !(flags & WABT_SYMBOL_FLAG_UNDEFINED),
            binding == SymbolBinding::Weak, binding == SymbolBinding::Local));
    }
    if (name.empty()) {
        return Result::Ok;
    }
    if (funcIndex >= wasmModule->funcs.size()) {
        printError("invalid function index: %" PRIindex, funcIndex);
        return Result::Error;
    }
    Func *func = wasmModule->funcs[funcIndex];
    if (!func->name.empty()) {
        // The name section has already named this function.
        return Result::Ok;
    }
    std::string dollarName =
        getUniqueName(&wasmModule->func_bindings, makeDollarName(name));
    func->name = dollarName;
    wasmModule->func_bindings.emplace(dollarName, Binding(funcIndex));
    return Result::Ok;
}

Result BinaryReaderIR::OnGlobalSymbol(Index index, uint32_t flags,
                                      StringRef name, Index globalIndex) {
    assert(index == wasmModule->symbols.size());
    SymbolBinding binding =
        static_cast<SymbolBinding>(flags & WABT_SYMBOL_MASK_BINDING);
    wasmModule->symbols.push_back(std::make_unique<WASMModule::GlobalSymbol>(
        name.str(), wasmModule, !(flags & WABT_SYMBOL_FLAG_UNDEFINED),
        binding == SymbolBinding::Weak, binding == SymbolBinding::Local));
    return setGlobalName(globalIndex, name);
}

Result BinaryReaderIR::OnSectionSymbol(Index index, uint32_t flags,
                                       Index sectionIndex) {
    assert(index == wasmModule->symbols.size());
    SymbolBinding binding =
        static_cast<SymbolBinding>(flags & WABT_SYMBOL_MASK_BINDING);
    wasmModule->symbols.push_back(std::make_unique<WASMModule::SectionSymbol>(
        std::string("???section???"), wasmModule,
        !(flags & WABT_SYMBOL_FLAG_UNDEFINED), binding == SymbolBinding::Weak,
        binding == SymbolBinding::Local));
    return Result::Ok;
}

Result BinaryReaderIR::OnTagSymbol(Index index, uint32_t flags, StringRef name,
                                   Index tagIndex) {
    assert(index == wasmModule->symbols.size());
    SymbolBinding binding =
        static_cast<SymbolBinding>(flags & WABT_SYMBOL_MASK_BINDING);
    wasmModule->symbols.push_back(std::make_unique<WASMModule::TagSymbol>(
        name.str(), wasmModule, !(flags & WABT_SYMBOL_FLAG_UNDEFINED),
        binding == SymbolBinding::Weak, binding == SymbolBinding::Local));
    if (name.empty()) {
        return Result::Ok;
    }
    if (tagIndex >= wasmModule->tags.size()) {
        printError("invalid tag index: %" PRIindex, tagIndex);
        return Result::Error;
    }
    Tag *tag = wasmModule->tags[tagIndex];
    std::string dollarName =
        getUniqueName(&wasmModule->tag_bindings, makeDollarName(name));
    tag->name = dollarName;
    wasmModule->tag_bindings.emplace(dollarName, Binding(tagIndex));
    return Result::Ok;
}

Result BinaryReaderIR::OnTableSymbol(Index index, uint32_t flags,
                                     StringRef name, Index tableIndex) {
    assert(index == wasmModule->symbols.size());
    SymbolBinding binding =
        static_cast<SymbolBinding>(flags & WABT_SYMBOL_MASK_BINDING);
    wasmModule->symbols.push_back(std::make_unique<WASMModule::TableSymbol>(
        name.str(), wasmModule, !(flags & WABT_SYMBOL_FLAG_UNDEFINED),
        binding == SymbolBinding::Weak, binding == SymbolBinding::Local));
    return setTableName(tableIndex, name);
}

Result BinaryReaderIR::OnGenericCustomSection(StringRef name, const void *data,
                                              Offset size) {
    Custom custom = Custom(name);
    custom.data.resize(size);
    if (size > 0) {
        memcpy(custom.data.data(), data, size);
    }
    wasmModule->customs.push_back(std::move(custom));
    return Result::Ok;
}

} // end anonymous namespace

Result readBinaryIr(const char *filename, const void *data, size_t size,
                    const ReadBinaryOptions &options, Errors *errors,
                    WASMModule *outModule) {
    BinaryReaderIR reader(outModule, filename, errors);
    return ReadBinary(data, size, &reader, options);
}

} // namespace wabt
