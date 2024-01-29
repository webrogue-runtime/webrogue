#pragma once
#include "ApiObject.hpp"
#include "ModsRuntime.hpp"

namespace webrogue {
namespace core {
template <typename Obj> struct ObjectExtractor;

template <> struct ObjectExtractor<ApiObject> {
    using Obj = core::ApiObject;
    static Obj *get(core::ModsRuntime *runtime) {
        return &(runtime->apiObject);
    }
};

template <> struct ObjectExtractor<WASIObject> {
    using Obj = core::WASIObject;
    static Obj *get(core::ModsRuntime *runtime) {
        return &(runtime->wasiObject);
    }
};
} // namespace core
} // namespace webrogue
