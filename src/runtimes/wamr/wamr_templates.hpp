#include "../../../external/wamr/core/iwasm/include/wasm_export.h"
#include "../../core/ModsRuntime.hpp"
#include "../../core/ObjectExtractor.hpp"
#include <stdint.h>
#include <vector>

namespace webrogue {
namespace runtimes {
namespace wamr {
template <char signature> struct TypeToSignatureImpl {
    static const char value = signature;
};
template <typename T, typename = void> struct TypeToSignature;

template <>
struct TypeToSignature<core::WASMRawI32> : TypeToSignatureImpl<'i'> {};
template <>
struct TypeToSignature<core::WASMRawU32> : TypeToSignatureImpl<'i'> {};
template <>
struct TypeToSignature<core::WASMRawI64> : TypeToSignatureImpl<'I'> {};
template <>
struct TypeToSignature<core::WASMRawU64> : TypeToSignatureImpl<'I'> {};
template <>
struct TypeToSignature<core::WASMRawF32> : TypeToSignatureImpl<'f'> {};
template <>
struct TypeToSignature<core::WASMRawF64> : TypeToSignatureImpl<'F'> {};

template <typename Ret, typename... Args> struct SignatureHelper;
template <typename... Args> struct SignatureHelper<void, Args...> {
    static char *getSignature() {
        static char result[] = {'(', TypeToSignature<Args>::value..., ')', 0,
                                0};
        return result;
    }
};
template <typename Ret, typename... Args> struct SignatureHelper {
    static char *getSignature() {
        static char result[] = {'(', TypeToSignature<Args>::value..., ')',
                                TypeToSignature<Ret>::value, 0};
        return result;
    }
};

template <typename Func, Func func> struct FuncWrapper;
template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct FuncWrapper<Ret (Obj::*)(Args...), method> {
    using Func = Ret (Obj::*)(Args...);
    static Ret wrappedFunc(wasm_exec_env_t execEnv, Args... args) {
        auto *runtime =
            (webrogue::core::ModsRuntime *)wasm_runtime_get_user_data(execEnv);
        return (core::ObjectExtractor<Obj>::get(runtime)->*method)(args...);
    }
};

template <typename Func, Func func> struct FuncLinker;

template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct FuncLinker<Ret (Obj::*)(Args...), method> {
    static NativeSymbol nativeSymbol(const char *funcName) {
        return {
            funcName,
            (void *)(&FuncWrapper<Ret (Obj::*)(Args...), method>::wrappedFunc),
            SignatureHelper<Ret, Args...>::getSignature()};
    }
};
} // namespace wamr
} // namespace runtimes
} // namespace webrogue
