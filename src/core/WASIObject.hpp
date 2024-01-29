#pragma once
#include "Config.hpp"
#include "ResourceStorage.hpp"
#include "VFS.hpp"
#include "wasm_types.hpp"
#include <cstdint>

struct uvwasi_s;

namespace webrogue {
namespace core {
class ModsRuntime;

class WASIObject {
public:
    uvwasi_s *uvwasi;
    VFS vfs;
    ModsRuntime *runtime;
    WASIObject(ModsRuntime *pRuntime, ResourceStorage *resourceStorage,
               Config *config);
    ~WASIObject();

#define WASI_FUNCTION(RET_TYPE, NAME, ARGS) RET_TYPE NAME ARGS;
#include "wasi_functions.def"
#undef WASI_FUNCTION
};
} // namespace core
} // namespace webrogue
