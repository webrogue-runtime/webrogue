#pragma once

#include "../ir.h"
#include "../result.h"

using namespace std;
namespace wabt {

class ImportSynthesizer {
public:
    vector<Import *> imports;
    vector<ImportedFunc> importedFuncs;

    Result synthesizeImports(const vector<unique_ptr<WASMModule>> *inputs,
                             const map<string, WASMModule::Symbol *> *symbols,
                             const vector<ImplementedFunc> *implementedFuncs);

private:
    void appendFunctionImport(const FuncImport *import, Index loadOrder,
                              const vector<ImplementedFunc> *implementedFuncs);
    void appendImport(Import *import);
};
} // namespace wabt