#pragma once
#include "../../mods/core/include/core.h"
#include "Config.hpp"
#include "DB.hpp"
#include "Terminal.hpp"
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
    Terminal *terminal;
    Config const *config;
    ApiObject(ModsRuntime *pRuntime, Config const *pConfig);

#define WR_API_FUNCTION(RET_TYPE, NAME, ARGS) RET_TYPE NAME ARGS;
#include "../../mods/core/include/common/wr_api_functions.def"
#undef WR_API_FUNCTION
};
} // namespace core
} // namespace webrogue
