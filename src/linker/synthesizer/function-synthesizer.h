#pragma once

#include "../ir.h"
#include "../result.h"
#include "import-synthesizer.h"
#include <functional>

using namespace std;
namespace wabt {
class FunctionSynthesizer {
public:
    Result synthesizeFunctions(
        const vector<unique_ptr<WASMModule>> *inputs,
        const map<string, WASMModule::Symbol *> *symbols,
        const ImportSynthesizer *importSynthesizer,
        vector<ImplementedFunc> *implementedFuncs,
        function<Result(Index originalIndex, const WASMModule *origin,
                        Index *outIndex)>
            relocateFunc);
};
} // namespace wabt
