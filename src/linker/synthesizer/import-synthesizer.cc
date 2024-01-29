#include "import-synthesizer.h"

#include "../cast.h"

namespace wabt {

Result ImportSynthesizer::synthesizeImports(
    const vector<unique_ptr<WASMModule>> *inputs,
    const map<string, WASMModule::Symbol *> *symbols,
    const vector<ImplementedFunc> *implementedFuncs) {
    for (auto &input : *inputs) {
        Index funcIndex = 0;
        for (auto &import : input->imports) {
            if (import->kind() == ExternalKind::Table) {
                assert(import->field_name == "__indirect_function_table");
            } else if (import->kind() == ExternalKind::Func) {
                for (auto &symbol : input->symbols)
                    if (symbol->kind() == WASMModule::Symbol::Kind::Func) {
                        auto *funcSymbol =
                            cast<WASMModule::FuncSymbol>(symbol.get());
                        if (funcSymbol->original_func_index == funcIndex) {
                            auto *actualSymbol = symbols->at(funcSymbol->name);
                            if (actualSymbol) {
                                funcSymbol =
                                    cast<WASMModule::FuncSymbol>(actualSymbol);
                                if (!funcSymbol->defined && !funcSymbol->weak)
                                    appendFunctionImport(
                                        cast<FuncImport>(import),
                                        actualSymbol->load_order,
                                        implementedFuncs);
                            }
                        }
                    }
                funcIndex++;
            } else {
                appendImport(import);
            }
        }
    }
    struct ImportComparator {
        bool operator()(Import *&x, Import *&y) const {
            return x->field_name > y->field_name;
        }
    };
    sort(imports.begin(), imports.end(), ImportComparator());
    struct ImportedFuncComparator {
        bool operator()(ImportedFunc &x, ImportedFunc &y) const {
            return x.loadOrder < y.loadOrder;
        }
    };
    sort(importedFuncs.begin(), importedFuncs.end(), ImportedFuncComparator());

    return Result::Ok;
}
void ImportSynthesizer::appendFunctionImport(
    const FuncImport *import, Index loadOrder,
    vector<ImplementedFunc> const *implementedFuncs) {
    for (auto &importedFunc : importedFuncs)
        if (importedFunc.fieldName == import->field_name &&
            importedFunc.moduleName == import->module_name)
            return;

    if (import->module_name == "env")
        for (auto &implementedFunc : *implementedFuncs)
            if ("$" + import->field_name == implementedFunc.name)
                return;

    ImportedFunc importedFunc;
    importedFunc.fieldName = import->field_name;
    importedFunc.moduleName = import->module_name;
    importedFunc.signature = import->func.decl.sig;
    importedFunc.loadOrder = loadOrder;
    importedFuncs.push_back(importedFunc);
}
void ImportSynthesizer::appendImport(Import *import) {
    for (auto &oldImport : imports)
        if (oldImport->module_name == import->module_name &&
            oldImport->field_name == import->field_name)
            return;
    imports.push_back(import);
}
} // namespace wabt