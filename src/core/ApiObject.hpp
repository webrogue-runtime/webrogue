#pragma once
#include "Config.hpp"
#include "ConsoleWriter.hpp"
#include "DB.hpp"
#include "Output.hpp"
#include "wasm_types.hpp"
#include <cstdint>
#include <memory>
#include <vector>

namespace webrogue {
namespace core {
class ModsRuntime;

class ApiObject {
public:
    ModsRuntime *runtime;
    std::unique_ptr<DB> db;
    Output *output;
    ConsoleWriter *consoleWriter;
    Config *config;
    std::vector<webrogue_raw_event> rawEvents;
    ApiObject(ModsRuntime *pRuntime, Config *pConfig);

#define WR_API_FUNCTION(RET_TYPE, NAME, ARGS) RET_TYPE NAME ARGS;
#include "../../mods/core/include/common/wr_api_functions.def"
#undef WR_API_FUNCTION
};
} // namespace core
} // namespace webrogue
