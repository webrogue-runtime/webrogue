#include "synthesizer.h"

using namespace std;
namespace wabt {
void Synthesizer::appendSignature(FuncSignature signature) {
    for (auto &oldSignature : signatures)
        if (oldSignature == signature)
            return;
    signatures.push_back(signature);
}

Result Synthesizer::synthesizeSignatures() {
    for (auto &input : *inputs) {
        for (auto &reloc : input->relocs) {
            if (getRelocSemantic(reloc.type) != RelocSemantic::TypeIndex)
                continue;
            if (reloc.section_index != input->code_section_index)
                continue;
            TypeEntry *typeEntry = input->types[reloc.index];
            if (typeEntry->kind() != TypeEntryKind::Func)
                return Result::Error;

            appendSignature(cast<FuncType>(typeEntry)->sig);
        }
        for (auto &reloc : input->relocs) {
            if (getRelocSemantic(reloc.type) != RelocSemantic::TypeIndex)
                continue;
            if (reloc.section_index != input->data_section_index)
                continue;
            TypeEntry *typeEntry = input->types[reloc.index];
            if (typeEntry->kind() != TypeEntryKind::Func)
                return Result::Error;

            appendSignature(cast<FuncType>(typeEntry)->sig);
        }
        for (auto &reloc : input->relocs) {
            if (getRelocSemantic(reloc.type) != RelocSemantic::TypeIndex)
                continue;
            if (reloc.section_index == input->data_section_index ||
                reloc.section_index != input->code_section_index)
                continue;
            TypeEntry *typeEntry = input->types[reloc.index];
            if (typeEntry->kind() != TypeEntryKind::Func)
                return Result::Error;

            appendSignature(cast<FuncType>(typeEntry)->sig);
        }
    }
    for (auto importedFunc : importSynthesizer.importedFuncs)
        appendSignature(importedFunc.signature);
    for (auto &input : *inputs)
        for (Index funcIndex = input->num_func_imports;
             funcIndex < input->funcs.size(); funcIndex++)
            appendSignature(input->funcs[funcIndex]->decl.sig);

    appendSignature({}); // for __wasm_call_ctors
    return Result::Ok;
}

Result Synthesizer::synthesizeExports() {
    for (auto &symbol : symbols) {
        if (!symbol.second->exported)
            continue;
        if (symbol.second->kind() == WASMModule::Symbol::Kind::Func) {
            auto *funcSymbol = cast<WASMModule::FuncSymbol>(symbol.second);
            Index newIndex;
            CHECK_RESULT(relocateFunc(funcSymbol->original_func_index,
                                      funcSymbol->module_, &newIndex));
            auto &func =
                implementedFuncs[newIndex -
                                 importSynthesizer.importedFuncs.size()];
            func.isExported = true;
            func.exportName = symbol.second->name;
        }
    }
    return Result::Ok;
}
Result Synthesizer::relocateTable(Index originalIndex, const WASMModule *origin,
                                  Index *outIndex) {
    assert(originalIndex == 0);
    *outIndex = 0;
    return Result::Ok;
}
Result Synthesizer::relocateType(Index originalIndex, const WASMModule *origin,
                                 Index *outIndex) {
    TypeEntry *originalType = origin->types[originalIndex];
    if (originalType->kind() == TypeEntryKind::Func) {
        FuncType *originalFuncType = cast<FuncType>(originalType);
        for (Index typeIndex = 0; typeIndex < signatures.size(); typeIndex++) {
            auto signature = signatures[typeIndex];
            if (signature == originalFuncType->sig) {
                *outIndex = typeIndex;
                return Result::Ok;
            }
        }
    }
    return Result::Error;
}
Result Synthesizer::relocateFunc(Index originalIndex, const WASMModule *origin,
                                 Index *outIndex) {
    for (auto &symbol : origin->symbols) {
        if (symbol->kind() != WASMModule::Symbol::Kind::Func)
            continue;
        auto *funcSymbol = cast<WASMModule::FuncSymbol>(symbol.get());
        if (funcSymbol->original_func_index == originalIndex) {
            if (funcSymbol->binding_local)
                continue;
            *outIndex =
                cast<WASMModule::FuncSymbol>(symbols.at(funcSymbol->name))
                    ->new_func_index;
            if (*outIndex != 4294967295)
                return Result::Ok;
        }
    }
    for (Index index = 0; index < implementedFuncs.size(); index++) {
        auto &implementedFunc = implementedFuncs[index];
        if (implementedFunc.origin == origin &&
            implementedFunc.originFuncIndex == originalIndex) {
            *outIndex = index + importSynthesizer.importedFuncs.size();

            return Result::Ok;
        }
    }
    Index funcImportIndex = 0;
    FuncImport *funcImport = nullptr;
    for (Index importIndex = 0; importIndex < origin->imports.size();
         importIndex++) {
        Import *originalImport = origin->imports[importIndex];
        if (originalImport->kind() != ExternalKind::Func)
            continue;
        if (funcImportIndex != originalIndex) {
            funcImportIndex++;
            continue;
        }
        funcImport = cast<FuncImport>(originalImport);
        break;
    }
    if (!funcImport)
        return Result::Error;
    for (Index index = 0; index < importSynthesizer.importedFuncs.size();
         index++) {
        auto &importedFunc = importSynthesizer.importedFuncs[index];
        if (importedFunc.fieldName == funcImport->field_name &&
            importedFunc.moduleName == funcImport->module_name) {
            *outIndex = index;
            return Result::Ok;
        }
    }
    if (funcImport->module_name == "env") {
        auto *originalSymbol =
            cast<WASMModule::FuncSymbol>(symbols[funcImport->field_name]);
        for (auto symbol : symbols) {
            if (symbol.second->kind() != WASMModule::Symbol::Kind::Func)
                continue;
            auto *otherSymbol = cast<WASMModule::FuncSymbol>(symbol.second);
            if (otherSymbol->original_func_index !=
                originalSymbol->original_func_index)
                continue;
            if (otherSymbol->module_ != originalSymbol->module_)
                continue;
            auto funcName = otherSymbol->name;
            for (Index index = 0; index < implementedFuncs.size(); index++) {
                auto &implementedFunc = implementedFuncs[index];
                StringRef implementedFuncName = implementedFunc.name;
                if (!implementedFuncName.empty() &&
                    implementedFuncName[0] == '$')
                    implementedFuncName = implementedFuncName.substr(1);
                if (implementedFuncName == funcName) {
                    *outIndex = index + importSynthesizer.importedFuncs.size();

                    return Result::Ok;
                }
            }
        }
    }
    return Result::Error; // TODO error
}
Result Synthesizer::relocateTableElement(Index originalIndex,
                                         const WASMModule *origin,
                                         Index *outIndex) {
    if (origin->function_symbol_map.count(originalIndex)) {
        Index funcIndex;
        CHECK_RESULT(relocateFunc(origin->function_symbol_map.at(originalIndex),
                                  origin, &funcIndex));

        // I dont know what table to use, so let`s
        // pretend there is only __indirect_function_table
        for (Index elementIndex = 0; elementIndex < indirectFuncTable.size();
             elementIndex++) {
            if (indirectFuncTable[elementIndex] == funcIndex) {
                *outIndex = elementIndex + 1;
                return Result::Ok;
            }
        }
        *outIndex = 0;
        return Result::Ok; // TODO error
    }
    return Result::Error;
}
Result Synthesizer::relocateMemory(Index originalIndex,
                                   const WASMModule *origin, Address *out) {
    StringRef name = origin->data_symbol_name_map.at(originalIndex);

    if (name == "__dso_handle") {
        *out = dataStart;
        return Result::Ok;
    }
    if (name == "__data_end") {
        *out = dataEnd;
        return Result::Ok;
    }
    if (name == "__stack_low") {
        *out = stackStart;
        return Result::Ok;
    }
    if (name == "__stack_high") {
        *out = stackEnd;
        return Result::Ok;
    }
    if (name == "__heap_base") {
        *out = heapStart;
        return Result::Ok;
    }
    if (name == "__heap_end") {
        *out = heapEnd;
        return Result::Ok;
    }

    Address innerOffset =
        cast<WASMModule::DataSymbol>(origin->symbols[originalIndex].get())
            ->offset;

    for (auto &dataSegment : data) {
        if (!dataSegment.relativeSymbolOffsets.count(name.str()))
            continue;
        set<pair<const WASMModule *, Offset>> candidates =
            dataSegment.relativeSymbolOffsets[name.str()];
        Offset relativeSymbolOffset;
        if (candidates.size() == 1) {
            relativeSymbolOffset = candidates.begin()->second;
        } else { // local
            bool found = false;
            for (auto &candidate : candidates) {
                if (candidate.first == origin) {
                    found = true;
                    relativeSymbolOffset = candidate.second;
                    break;
                }
            }
            assert(found);
        }

        auto &offsetExpr = dataSegment.offset;
        assert(offsetExpr.size() == 1);

        for (auto &expr : offsetExpr) {
            assert(expr.type() == ExprType::Const);
            *out = cast<ConstExpr>(&expr)->const_.u32() + relativeSymbolOffset;
            *out += innerOffset;
            return Result::Ok;
        }
    }
    *out = 0;
    return Result::Ok; // TODO ?
}
Result Synthesizer::synthesizeTables() {
    for (auto &input : *inputs)
        for (RelocSectionKind section :
             {RelocSectionKind::Code, RelocSectionKind::Data,
              RelocSectionKind::Custom})
            for (auto &reloc : input->relocs) {
                RelocSectionKind actualSection;
                if (reloc.section_index == input->code_section_index)
                    actualSection = RelocSectionKind::Code;
                else if (reloc.section_index == input->data_section_index)
                    actualSection = RelocSectionKind::Data;
                else
                    actualSection = RelocSectionKind::Code;
                if (actualSection != section)
                    continue;
                if (getRelocSemantic(reloc.type) != RelocSemantic::TableIndex &&
                    getRelocSemantic(reloc.type) !=
                        RelocSemantic::TableIndexRel)
                    continue;
                Index newIndex;
                WASMModule::FuncSymbol *symbol = cast<WASMModule::FuncSymbol>(
                    input->symbols[reloc.index].get());
                // symbol =
                // cast<Module::FuncSymbol>(symbols->at(symbol->name));
                if (symbol->weak && !symbol->defined)
                    continue;
                CHECK_RESULT(relocateFunc(symbol->original_func_index,
                                          input.get(), &newIndex));
                bool alreadyContains = false;
                for (auto oldElement : indirectFuncTable)
                    if (oldElement == newIndex) {
                        alreadyContains = true;
                        break;
                    }
                if (!alreadyContains)
                    indirectFuncTable.push_back(newIndex);
            }

    return Result::Ok;
}
void Synthesizer::generalizeDataSymbolName(StringRef &name) {
    if (name[0] == '$')
        name = name.substr(1);
    if (name.substr(0, 6) == ".data.")
        name = name.substr(6);
    if (name.substr(0, 5) == ".bss.")
        name = name.substr(5);
    if (name.substr(0, 8) == ".rodata.")
        name = name.substr(8);
    // while (name[0] == '_')
    //   name = name.substr(1);
}
Result Synthesizer::synthesizeData() {
    Address memoryEnd = 0;
    stackStart = memoryEnd;
    memoryEnd += stackSize;
    stackEnd = memoryEnd;

    struct OutputSegment {
        vector<InputSegment> segments;
        Address alignmentLog2 = 0;
        bool bss = false;
        string name;
    };
    vector<OutputSegment> outputSegments;
    for (auto &input : *inputs) {
        for (Index originalDataSegmentIndex = 0;
             originalDataSegmentIndex < input->data_segments.size();
             originalDataSegmentIndex++) {
            auto *originalDataSegment =
                input->data_segments[originalDataSegmentIndex];
            InputSegment inputSegment;

            // Offset original_offset =
            //     cast<ConstExpr>(&original_data_segment->offset.front())
            //         ->const_.u32();
            inputSegment.original_positions = {
                make_pair(input.get(), originalDataSegmentIndex)};
            inputSegment.data = originalDataSegment->data;
            bool ignoredByComdat = false;
            for (auto &symbolCandidate : input->symbols) {
                if (symbolCandidate->kind() != WASMModule::Symbol::Kind::Data ||
                    !symbolCandidate->defined)
                    continue;
                auto *dataSymbolCandidate =
                    cast<WASMModule::DataSymbol>(symbolCandidate.get());
                if (dataSymbolCandidate->segment_index ==
                    originalDataSegmentIndex) {
                    inputSegment.origins_and_symbol_names.insert(
                        OriginsAndSymbolName(input.get(),
                                             dataSymbolCandidate->name, 0));
                    ignoredByComdat = dataSymbolCandidate->ignored_by_comdat;
                }
            }
            // if (ignored_by_comdat)
            //   continue;
            WASMModule::DataSegmentInfo dataSegmentInfo =
                input->data_segment_info[originalDataSegmentIndex];

            inputSegment.is_string = dataSegmentInfo.strings_flag &&
                                     dataSegmentInfo.alignment_log2 == 0;
            inputSegment.alignment_log2 = dataSegmentInfo.alignment_log2;

            OutputSegment *outputSegment = nullptr;
            for (auto &outputSegmentCandidate : outputSegments)
                if (outputSegmentCandidate.name == dataSegmentInfo.name) {
                    outputSegment = &outputSegmentCandidate;
                    break;
                }
            if (!outputSegment) {
                outputSegments.push_back(OutputSegment());
                outputSegment = &*outputSegments.rbegin();
                outputSegment->name = dataSegmentInfo.name;
                outputSegment->alignmentLog2 = 0;
            }
            outputSegment->alignmentLog2 = max(outputSegment->alignmentLog2,
                                               dataSegmentInfo.alignment_log2);

            outputSegment->segments.push_back(inputSegment);
            const string bssPrefix = ".bss";
            if (dataSegmentInfo.name.substr(0, bssPrefix.size()) == bssPrefix)
                outputSegment->bss = true;
        }
    }
    stable_sort(
        outputSegments.begin(), outputSegments.end(),
        [](const OutputSegment a, const OutputSegment b) {
            auto order = [](string name) {
                int group = 3;
                const string tdataPrefix = ".tdata";
                const string rodataPrefix = ".rodata";
                const string dataPrefix = ".data";
                const string bssPrefix = ".bss";
                if (name.substr(0, tdataPrefix.size()) == tdataPrefix)
                    group = 0;
                else if (name.substr(0, rodataPrefix.size()) == rodataPrefix)
                    group = 1;
                else if (name.substr(0, dataPrefix.size()) == dataPrefix)
                    group = 2;
                else if (name.substr(0, bssPrefix.size()) == bssPrefix)
                    group = 4;
                return group;
            };
            return order(a.name) < order(b.name);
        });
    for (auto &outputSegment : outputSegments) {
        vector<InputSegment> preStringsSegments;
        vector<InputSegment> stringsSegments;
        vector<InputSegment> postStringsSegments;
        for (auto &inputSegment : outputSegment.segments)
            if (inputSegment.is_string) {
                stringsSegments.push_back(inputSegment);
            } else if (stringsSegments.empty())
                preStringsSegments.push_back(inputSegment);
            else
                postStringsSegments.push_back(inputSegment);

        map<uint32_t, InputSegment> bucket;
        vector<uint32_t> hashOrder;
        for (auto &inputSegment : stringsSegments) {
            uint64_t rawHash = xxHash64(inputSegment.data);
            uint32_t hash = (rawHash & 0xffffffff) >> 1;
            if (bucket.count(hash)) {
                bucket[hash].original_positions.insert(
                    inputSegment.original_positions.begin(),
                    inputSegment.original_positions.end());
                bucket[hash].origins_and_symbol_names.insert(
                    inputSegment.origins_and_symbol_names.begin(),
                    inputSegment.origins_and_symbol_names.end());
                continue;
            }
            bucket[hash] = inputSegment;
            hashOrder.push_back(hash);
        }
        stringsSegments.clear();
        for (auto hash : hashOrder)
            stringsSegments.push_back(bucket[hash]);
        multikeySort(stringsSegments.begin(), stringsSegments.end(), 0);

        outputSegment.segments.clear();
        outputSegment.segments.insert(outputSegment.segments.end(),
                                      preStringsSegments.begin(),
                                      preStringsSegments.end());
        for (auto stringsSegment = stringsSegments.begin();
             stringsSegment != stringsSegments.end(); stringsSegment++) {
            if (stringsSegment != stringsSegments.begin()) {
                auto &prev = *outputSegment.segments.rbegin();
                if (prev.data.size() > stringsSegment->data.size()) {
                    Address padding =
                        prev.data.size() - stringsSegment->data.size();
                    if (memcmp(prev.data.data() + padding,
                               stringsSegment->data.data(),
                               stringsSegment->data.size()) == 0) {
                        for (auto originAndSymbolName :
                             stringsSegment->origins_and_symbol_names) {
                            prev.origins_and_symbol_names.emplace(
                                OriginsAndSymbolName(
                                    originAndSymbolName.origin,
                                    originAndSymbolName.symbol_name,
                                    originAndSymbolName.optimization_offset +
                                        padding));
                        }
                        prev.original_positions.insert(
                            stringsSegment->original_positions.begin(),
                            stringsSegment->original_positions.end());
                        continue;
                    }
                }
            }
            outputSegment.segments.push_back(*stringsSegment);
        }
        outputSegment.segments.insert(outputSegment.segments.end(),
                                      postStringsSegments.begin(),
                                      postStringsSegments.end());
    }

    dataStart = memoryEnd;
    for (auto &outputSegment : outputSegments) {
        Index pushedDataIndex = data.size();
        data.push_back(ExtendedDataSegment(outputSegment.name));
        ExtendedDataSegment &pushedDataSegment = *data.rbegin();

        pushedDataSegment.bss = outputSegment.bss;

        memoryEnd = aling(memoryEnd, 1 << outputSegment.alignmentLog2);
        pushedDataSegment.offset = {};
        pushedDataSegment.offset.push_back(
            make_unique<ConstExpr>(Const::I32(memoryEnd)));

        auto outputSegmentStartOffset = memoryEnd;
        for (auto &inputSegment : outputSegment.segments) {
            Address oldMemoryEnd = memoryEnd;
            memoryEnd = aling(memoryEnd, 1 << inputSegment.alignment_log2);
            for (Index i = 0; i < (memoryEnd - oldMemoryEnd); i++)
                pushedDataSegment.data.push_back(0);

            pushedDataSegment.data.insert(pushedDataSegment.data.end(),
                                          inputSegment.data.begin(),
                                          inputSegment.data.end());

            for (auto &originalPos : inputSegment.original_positions)
                dataRelocations[originalPos] = pushedDataIndex;
            for (auto &originAndSymbolName :
                 inputSegment.origins_and_symbol_names) {
                if (!pushedDataSegment.relativeSymbolOffsets.count(
                        originAndSymbolName.symbol_name))
                    pushedDataSegment.relativeSymbolOffsets
                        [originAndSymbolName.symbol_name] = {};
                pushedDataSegment
                    .relativeSymbolOffsets[originAndSymbolName.symbol_name]
                    .emplace(
                        make_pair(originAndSymbolName.origin,
                                  memoryEnd - outputSegmentStartOffset +
                                      originAndSymbolName.optimization_offset));
            }
            memoryEnd += inputSegment.data.size();
        }

        pushedDataSegment.memory_var = Var(0);
    }

    dataEnd = memoryEnd;
    memoryEnd = aling(memoryEnd, 16); // Head alignment
    heapStart = memoryEnd;
    memoryEnd = aling(memoryEnd, pageSize);
    heapEnd = memoryEnd;
    totalPages = memoryEnd / pageSize;

    return Result::Ok;
}
Result Synthesizer::relocateOne(RelocSemantic relocSemantic,
                                WASMModule::Reloc reloc,
                                const WASMModule *origin, Address *out) {
    switch (relocSemantic) {
    case RelocSemantic::FuncIndex: {
        Index outIndex;
        CHECK_RESULT(relocateFunc(origin->function_symbol_map.at(reloc.index),
                                  origin, &outIndex));
        *out = outIndex;
        return Result::Ok;
    }
    case RelocSemantic::TypeIndex: {
        Index outIndex;
        CHECK_RESULT(relocateType(reloc.index, origin, &outIndex));
        *out = outIndex;
        return Result::Ok;
    }
    case RelocSemantic::TableIndex: {
        Index outIndex;
        CHECK_RESULT(relocateTableElement(reloc.index, origin, &outIndex));
        *out = outIndex;
        return Result::Ok;
    }
    case RelocSemantic::MemoryAddress: {
        CHECK_RESULT(relocateMemory(reloc.index, origin, out));
        int64_t added = reloc.addend;
        if (added > 0x7fffffff)
            added = static_cast<int32_t>(static_cast<uint64_t>(added));
        *out += added;
        return Result::Ok;
    }
    case RelocSemantic::GlobalIndex: {
        *out = 0; // TODO
        return Result::Ok;
    }
    default:
        *out = 123456; // TODO  Error
        return Result::Error;
    }
}
Result Synthesizer::writeReloc(RelocForm relocForm, uint8_t *dataPtr,
                               uint8_t *end, Address value) {
    MemoryStream memoryStream;
    switch (relocForm) {
    case RelocForm::LEB: {
        WriteFixedU32Leb128(&memoryStream, value, "");
        break;
    }
    case RelocForm::SLEB: {
        WriteFixedS32Leb128(&memoryStream, value, "");
        break;
    }
    case RelocForm::I32: {
        memoryStream.WriteU32(value);
        break;
    }
    default:
        return Result::Error;
    }
    if (dataPtr + memoryStream.output_buffer().size() > end)
        return Result::Error;
    memcpy(dataPtr, memoryStream.output_buffer().data.data(),
           memoryStream.output_buffer().size());
    return Result::Ok;
}
Result Synthesizer::relocate() {
    for (auto &input : *inputs) {
        for (auto reloc : input->relocs) {
            bool isDataSection;
            if (reloc.section_index == input->code_section_index)
                isDataSection = false;
            else if (reloc.section_index == input->data_section_index)
                isDataSection = true;
            else
                continue;

            Address relocatedValue;
            interrupt();
            CHECK_RESULT(relocateOne(getRelocSemantic(reloc.type), reloc,
                                     input.get(), &relocatedValue));
            uint8_t *dataPtr = nullptr;
            uint8_t *endPtr = nullptr;
            if (isDataSection) {
                for (auto &offsetPair : input->data_segment_offsets) {
                    Address offset = offsetPair.second;
                    if (reloc.offset < offset)
                        continue;
                    auto &originalData = input->data_segments[offsetPair.first];
                    if (reloc.offset > (offset + originalData->data.size()))
                        continue;

                    auto index = make_pair(input.get(), offsetPair.first);
                    if (!dataRelocations.count(index))
                        continue;
                    auto &synthesizedData = data[dataRelocations.at(index)];

                    dataPtr =
                        synthesizedData.data.data() + reloc.offset - offset;
                    endPtr = synthesizedData.data.data() + synthesizedData.data.size();
                    break;
                }
            } else {
                dataPtr = input->code.data() + reloc.offset;
                endPtr = input->code.data() + input->code.size();
            }
            if (!dataPtr || !endPtr)
                continue;
            CHECK_RESULT(writeReloc(getRelocForm(reloc.type), dataPtr, endPtr,
                                    relocatedValue));
        }
    }
    return Result::Ok;
}
Result Synthesizer::synthesize(std::vector<unique_ptr<WASMModule>> *inputs,
                               map<string, WASMModule::Symbol *> *symbols,
                               WASMModule *outputModule) {
    this->symbols = *symbols;
    this->inputs = inputs;

    interrupt();
    CHECK_RESULT(importSynthesizer.synthesizeImports(inputs, symbols,
                                                     &implementedFuncs));
    interrupt();
    CHECK_RESULT(functionSynthesizer.synthesizeFunctions(
        inputs, symbols, &importSynthesizer, &implementedFuncs,
        [this](Index originalIndex, const WASMModule *origin, Index *outIndex) {
            return relocateFunc(originalIndex, origin, outIndex);
        }));
    interrupt();
    CHECK_RESULT(synthesizeSignatures());
    interrupt();
    CHECK_RESULT(synthesizeExports());
    interrupt();
    CHECK_RESULT(synthesizeTables());
    interrupt();
    CHECK_RESULT(synthesizeData());
    interrupt();

    CHECK_RESULT(relocate());

    for (auto &signature : signatures) {
        auto typeField = make_unique<TypeModuleField>();
        auto funcEntry = make_unique<FuncType>();
        funcEntry->sig = signature;
        typeField->type = std::move(funcEntry);
        outputModule->AppendField(std::move(typeField));
    }
    for (auto importedFunc : importSynthesizer.importedFuncs) {
        auto importField = make_unique<ImportModuleField>();
        auto import = make_unique<FuncImport>(importedFunc.fieldName);
        import->func.decl.sig = importedFunc.signature;
        importField->import = std::move(import);
        importField->import->field_name = importedFunc.fieldName.str();
        importField->import->module_name = importedFunc.moduleName.str();
        outputModule->AppendField(std::move(importField));
    }
    {
        auto memoryField = make_unique<MemoryModuleField>();
        memoryField->memory.page_limits.initial = totalPages;
        outputModule->AppendField(std::move(memoryField));
        auto exportField = make_unique<ExportModuleField>();
        exportField->export_.kind = ExternalKind::Memory;
        exportField->export_.name = "memory";
        exportField->export_.var = Var(0);
        outputModule->AppendField(std::move(exportField));
    }
    {
        auto globalField = make_unique<GlobalModuleField>();
        globalField->global.type = Type::I32;
        globalField->global.mutable_ = true;
        globalField->global.init_expr = {};
        globalField->global.init_expr.push_back(
            make_unique<ConstExpr>(Const::I32(stackSize)));
        outputModule->AppendField(std::move(globalField));
    }
    for (Index implementedFuncIndex = 0;
         implementedFuncIndex < implementedFuncs.size();
         implementedFuncIndex++) {
        auto &implementedFunc = implementedFuncs[implementedFuncIndex];
        auto funcField = make_unique<FuncModuleField>();
        funcField->func.name = implementedFunc.name;
        funcField->func.decl.sig = implementedFunc.signature;
        outputModule->AppendField(std::move(funcField));
        if (implementedFunc.isExported) {
            auto exportField = make_unique<ExportModuleField>();
            exportField->export_.kind = ExternalKind::Func;
            exportField->export_.name = implementedFunc.exportName;
            exportField->export_.var = Var(
                implementedFuncIndex + importSynthesizer.importedFuncs.size());
            outputModule->AppendField(std::move(exportField));
        }
    }
    {
        auto tableField = make_unique<TableModuleField>();
        tableField->table = Table("__indirect_function_table");
        tableField->table.elem_type = Type::Enum::FuncRef;
        tableField->table.elem_limits.has_max = true;
        tableField->table.elem_limits.max =
            tableField->table.elem_limits.initial =
                1 + indirectFuncTable.size();
        outputModule->AppendField(std::move(tableField));
        auto elementField = make_unique<ElemSegmentModuleField>();
        elementField->elem_segment.table_var = Var(0);
        elementField->elem_segment.elem_type = Type::FuncRef;
        elementField->elem_segment.offset = {};
        elementField->elem_segment.offset.push_back(
            make_unique<ConstExpr>(Const::I32(1)));
        elementField->elem_segment.elem_exprs.clear();
        for (auto &element : indirectFuncTable) {
            elementField->elem_segment.elem_exprs.push_back({});
            ExprList &exprList = elementField->elem_segment.elem_exprs.back();
            exprList.push_front(make_unique<RefFuncExpr>(Var(element)));
        }
        outputModule->AppendField(std::move(elementField));
    }
    for (auto implementedFunc : implementedFuncs) {
        if (!implementedFunc.syntesized)
            break; // synthesized funcs are always before other implemented
        outputModule->code.insert(outputModule->code.end(),
                                  implementedFunc.syntesizedCode.begin(),
                                  implementedFunc.syntesizedCode.end());
    }
    for (auto &input : *inputs) {
        outputModule->code.insert(
            outputModule->code.end(),
            std::next(input->code.begin(), input->code_count_size),
            input->code.end());
    }
    for (auto &dataSegment : data) {
        if (dataSegment.bss)
            continue;
        auto dataField = make_unique<DataSegmentModuleField>();
        dataField->data_segment = std::move(dataSegment);
        outputModule->AppendField(std::move(dataField));
    }
    return Result::Ok;
}
} // namespace wabt