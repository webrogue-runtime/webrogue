#include "function-synthesizer.h"
#include "../cast.h"
#include "../stream.h"
#include <cstdlib>

namespace wabt {

Result FunctionSynthesizer::synthesizeFunctions(
    const vector<unique_ptr<WASMModule>> *inputs,
    const map<string, WASMModule::Symbol *> *symbols,
    const ImportSynthesizer *importSynthesizer,
    vector<ImplementedFunc> *implementedFuncs,
    function<Result(Index originalIndex, const WASMModule *origin,
                    Index *outIndex)>
        relocateFunc) {
    {
        ImplementedFunc callCtorsFunc;
        callCtorsFunc.signature = {};
        callCtorsFunc.exportName = "__wasm_call_ctors";
        callCtorsFunc.isExported = true;
        callCtorsFunc.syntesized = true;
        implementedFuncs->push_back(callCtorsFunc);
    }
    {
        set<WASMModule::FuncSymbol *> weakUndefinedFuncSymbols;
        for (auto &symbolPair : *symbols)
            if (symbolPair.second->kind() == WASMModule::Symbol::Kind::Func &&
                symbolPair.second->weak && !symbolPair.second->defined)
                weakUndefinedFuncSymbols.insert(
                    cast<WASMModule::FuncSymbol>(symbolPair.second));
        for (auto *funcSymbol : weakUndefinedFuncSymbols) {
            ImplementedFunc unreachableFunc;
            unreachableFunc.signature =
                funcSymbol->module_->funcs[funcSymbol->original_func_index]
                    ->decl.sig;

            unreachableFunc.syntesized = true;
            unreachableFunc.syntesizedCode.push_back(0x03);
            unreachableFunc.syntesizedCode.push_back(0x00);
            unreachableFunc.syntesizedCode.push_back(0x00);
            unreachableFunc.syntesizedCode.push_back(0x0b);

            unreachableFunc.name = funcSymbol->name;

            funcSymbol->new_func_index =
                importSynthesizer->importedFuncs.size() +
                implementedFuncs->size();
            implementedFuncs->push_back(unreachableFunc);
        }
    }

    for (auto &input : *inputs)
        for (Index funcIndex = input->num_func_imports;
             funcIndex < input->funcs.size(); funcIndex++) {
            auto &originalFunc = input->funcs[funcIndex];
            ImplementedFunc func;

            bool foundName = false;
            for (auto &symbol : input->symbols) {
                if (symbol->kind() != WASMModule::Symbol::Kind::Func)
                    continue;
                auto *funcSymbol = cast<WASMModule::FuncSymbol>(symbol.get());
                if (funcSymbol->defined &&
                    funcSymbol->original_func_index == funcIndex) {
                    foundName = true;
                    func.name = funcSymbol->name;
                    funcSymbol->new_func_index =
                        importSynthesizer->importedFuncs.size() +
                        implementedFuncs->size();
                    break;
                }
            }
            if (!foundName)
                func.name = originalFunc->name;

            func.signature = originalFunc->decl.sig;
            func.origin = input.get();
            func.originFuncIndex = funcIndex;
            implementedFuncs->push_back(func);
        }

    {
        vector<pair<Index, WASMModule::Symbol *>> initFuncs;
        for (auto &input : *inputs)
            for (auto &initFunc : input->init_fimctions)
                initFuncs.push_back(initFunc);
        struct InitFuncComparator {
            bool operator()(pair<Index, WASMModule::Symbol *> &x,
                            pair<Index, WASMModule::Symbol *> &y) const {
                return x.first < y.first;
            }
        };
        sort(initFuncs.begin(), initFuncs.end(), InitFuncComparator());
        MemoryStream callCtorsBodyStream;
        callCtorsBodyStream.WriteU8(0); // num locals
        for (auto &initFunc : initFuncs) {
            auto *funcSymbol = cast<WASMModule::FuncSymbol>(initFunc.second);
            Index funcIndex;
            CHECK_RESULT(relocateFunc(funcSymbol->original_func_index,
                                      funcSymbol->module_, &funcIndex));
            FuncSignature signature;
            if (funcIndex < importSynthesizer->importedFuncs.size())
                signature = implementedFuncs->at(funcIndex).signature;
            else
                signature = implementedFuncs
                                ->at(funcIndex -
                                     importSynthesizer->importedFuncs.size())
                                .signature;

            callCtorsBodyStream.WriteU8(0x10);
            WriteU32Leb128(&callCtorsBodyStream, funcIndex, "");

            for (Index returnIndex = 0; returnIndex < signature.GetNumResults();
                 returnIndex++)
                callCtorsBodyStream.WriteU8(0x1a);
        }
        callCtorsBodyStream.WriteU8(0x0b);
        MemoryStream callCtorsStream;
        WriteU32Leb128(&callCtorsStream,
                       callCtorsBodyStream.output_buffer().size(), "");
        callCtorsStream.WriteData(
            callCtorsBodyStream.output_buffer().data.data(),
            callCtorsBodyStream.output_buffer().size(), "");

        implementedFuncs->at(0).syntesizedCode =
            callCtorsStream.output_buffer().data;
    }

    return Result::Ok;
}
} // namespace wabt
