#pragma once

#include "../../common/tuple_utils.hpp"
#include "../../core/ModsRuntime.hpp"
#include "../../core/ObjectExtractor.hpp"
#include "wasm.h"
#include <cstdint>
#include <cstring>
#include <map>
#include <string>

namespace webrogue {
namespace runtimes {
namespace wasm_c_api {

template <typename Ret, typename Func, typename ArgsTuple> struct ReturnHelper;
template <typename Func, typename ArgsTuple>
struct ReturnHelper<void, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     wasm_val_vec_t *out) {
        call(func, args);
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawI32, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     wasm_val_vec_t *results) {
        results->data[0].kind = WASM_I32;
        results->data[0].of.i32 = call(func, args).get();
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawU32, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     wasm_val_vec_t *results) {
        results->data[0].kind = WASM_I32;
        results->data[0].of.i32 = call(func, args).get();
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawI64, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     wasm_val_vec_t *results) {
        results->data[0].kind = WASM_I64;
        results->data[0].of.i64 = call(func, args).get();
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawU64, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     wasm_val_vec_t *results) {
        results->data[0].kind = WASM_I64;
        results->data[0].of.i64 = call(func, args).get();
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawF32, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     wasm_val_vec_t *results) {
        results->data[0].kind = WASM_F32;
        results->data[0].of.f32 = call(func, args).get();
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawF64, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     wasm_val_vec_t *results) {
        results->data[0].kind = WASM_F64;
        results->data[0].of.f64 = call(func, args).get();
    }
};

template <typename Func, Func func> struct SecondaryFuncWrapper;
template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct SecondaryFuncWrapper<Ret (Obj::*)(Args...), method> {
    static inline Ret wrappedFunc(void *data, Args... args) {
        return (
            core::ObjectExtractor<Obj>::get((webrogue::core::ModsRuntime *)data)
                ->*method)(args...);
    }
};

template <typename T> struct arg_stack_helper;

template <> struct arg_stack_helper<core::WASMRawI32> {
    static inline void argFromStack(core::WASMRawI32 &dest,
                                    const wasm_val_vec_t *args, size_t argI) {
        dest = core::WASMRawI32::make(args->data[argI].of.i32);
    }
};
template <> struct arg_stack_helper<core::WASMRawU32> {
    static inline void argFromStack(core::WASMRawU32 &dest,
                                    const wasm_val_vec_t *args, size_t argI) {
        dest = core::WASMRawU32::make(args->data[argI].of.i32);
    }
};
template <> struct arg_stack_helper<core::WASMRawI64> {
    static inline void argFromStack(core::WASMRawI64 &dest,
                                    const wasm_val_vec_t *args, size_t argI) {
        dest = core::WASMRawI64::make(args->data[argI].of.i64);
    }
};
template <> struct arg_stack_helper<core::WASMRawU64> {
    static inline void argFromStack(core::WASMRawU64 &dest,
                                    const wasm_val_vec_t *args, size_t argI) {
        dest = core::WASMRawU64::make(args->data[argI].of.i64);
    }
};
template <> struct arg_stack_helper<core::WASMRawF32> {
    static inline void argFromStack(core::WASMRawF32 &dest,
                                    const wasm_val_vec_t *args, size_t argI) {
        dest = core::WASMRawF32::make(args->data[argI].of.f32);
    }
};
template <> struct arg_stack_helper<core::WASMRawF64> {
    static inline void argFromStack(core::WASMRawF64 &dest,
                                    const wasm_val_vec_t *args, size_t argI) {
        dest = core::WASMRawF64::make(args->data[argI].of.f64);
    }
};

template <typename T>
void argFromStack(T &dest, const wasm_val_vec_t *args, size_t argI) {
    arg_stack_helper<T>::argFromStack(dest, args, argI);
}

template <typename... Args>
static inline void getArgsFromStack(const wasm_val_vec_t *args,
                                    std::tuple<Args...> &tuple) {
    size_t argI = 0;

    tupleForEach(tuple, [&](auto &item) {
        argFromStack(item, args, argI);
        argI++;
    });
}

template <typename Func, Func func> struct PrimaryFuncWrapper;
template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct PrimaryFuncWrapper<Ret (Obj::*)(Args...), method> {
    static inline wasm_trap_t *wrappedFunc(void *env,
                                           const wasm_val_vec_t *args,
                                           wasm_val_vec_t *results) {
        std::tuple<Args...> parsedArgs;

        constexpr auto f =
            SecondaryFuncWrapper<Ret (Obj::*)(Args...), method>::wrappedFunc;

        getArgsFromStack(args, parsedArgs);
        std::tuple<void *, Args...> nargs =
            std::tuple_cat(std::make_tuple(env), parsedArgs);
        ReturnHelper<Ret, decltype(f),
                     std::tuple<void *, Args...>>::callAndReturn(f, nargs,
                                                                 results);
        return NULL;
    }
};

template <typename T, typename = void> struct TypeToSignature;

template <> struct TypeToSignature<core::WASMRawI32> {
    static inline wasm_valtype_t *genType() {
        return wasm_valtype_new_i32();
    }
};
template <> struct TypeToSignature<core::WASMRawU32> {
    static inline wasm_valtype_t *genType() {
        return wasm_valtype_new_i32();
    }
};
template <> struct TypeToSignature<core::WASMRawI64> {
    static inline wasm_valtype_t *genType() {
        return wasm_valtype_new_i64();
    }
};
template <> struct TypeToSignature<core::WASMRawU64> {
    static inline wasm_valtype_t *genType() {
        return wasm_valtype_new_i64();
    }
};
template <> struct TypeToSignature<core::WASMRawF32> {
    static inline wasm_valtype_t *genType() {
        return wasm_valtype_new_f32();
    }
};
template <> struct TypeToSignature<core::WASMRawF64> {
    static inline wasm_valtype_t *genType() {
        return wasm_valtype_new_f64();
    }
};

template <typename... Args> struct ArgumentSignatureHelper {
    static void getSignature(wasm_valtype_vec_t *resultsVec) {
        wasm_valtype_t *results[] = {TypeToSignature<Args>::genType()...};
        size_t count = sizeof(results) / sizeof(wasm_valtype_t *);
        wasm_valtype_t **copiedResults = new wasm_valtype_t *[count];
        memcpy(copiedResults, results, sizeof(wasm_valtype_t *) * count);

        wasm_valtype_vec_new(resultsVec, count, copiedResults);
    }
};

template <typename Ret> struct ResultSignatureHelper;

template <> struct ResultSignatureHelper<void> {
    static void getSignature(wasm_valtype_vec_t *resultsVec) {
        wasm_valtype_vec_new_empty(resultsVec);
    }
};

template <> struct ResultSignatureHelper<core::WASMRawI32> {
    static void getSignature(wasm_valtype_vec_t *resultsVec) {
        wasm_valtype_t *results[1] = {wasm_valtype_new_i32()};
        wasm_valtype_vec_new(resultsVec, 1, results);
    }
};
template <> struct ResultSignatureHelper<core::WASMRawU32> {
    static void getSignature(wasm_valtype_vec_t *resultsVec) {
        wasm_valtype_t *results[1] = {wasm_valtype_new_i32()};
        wasm_valtype_vec_new(resultsVec, 1, results);
    }
};
template <> struct ResultSignatureHelper<core::WASMRawI64> {
    static void getSignature(wasm_valtype_vec_t *resultsVec) {
        wasm_valtype_t *results[1] = {wasm_valtype_new_i64()};
        wasm_valtype_vec_new(resultsVec, 1, results);
    }
};
template <> struct ResultSignatureHelper<core::WASMRawU64> {
    static void getSignature(wasm_valtype_vec_t *resultsVec) {
        wasm_valtype_t *results[1] = {wasm_valtype_new_i64()};
        wasm_valtype_vec_new(resultsVec, 1, results);
    }
};
template <> struct ResultSignatureHelper<core::WASMRawF32> {
    static void getSignature(wasm_valtype_vec_t *resultsVec) {
        wasm_valtype_t *results[1] = {wasm_valtype_new_f32()};
        wasm_valtype_vec_new(resultsVec, 1, results);
    }
};
template <> struct ResultSignatureHelper<core::WASMRawF64> {
    static void getSignature(wasm_valtype_vec_t *resultsVec) {
        wasm_valtype_t *results[1] = {wasm_valtype_new_f64()};
        wasm_valtype_vec_new(resultsVec, 1, results);
    }
};

template <typename Func, Func func> struct FuncLinker;

template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct FuncLinker<Ret (Obj::*)(Args...), method> {
    inline static void link(std::string moduleName, std::string functionName,
                            wasm_store_t *store, core::ModsRuntime *runtime,
                            std::map<std::string, wasm_extern_t *> &imports) {
        constexpr auto wrappedFunc =
            &(PrimaryFuncWrapper<Ret (Obj::*)(Args...), method>::wrappedFunc);

        wasm_valtype_vec_t params;
        ArgumentSignatureHelper<Args...>::getSignature(&params);
        wasm_valtype_vec_t results;
        ResultSignatureHelper<Ret>::getSignature(&results);

        wasm_functype_t *wrappedFuncType = wasm_functype_new(&params, &results);
        wasm_func_t *wrappedWasmFuncType = wasm_func_new_with_env(
            store, wrappedFuncType, wrappedFunc, runtime, NULL);

        wasm_functype_delete(wrappedFuncType);
        imports[moduleName + "." + functionName] =
            wasm_func_as_extern(wrappedWasmFuncType);
    }
};
} // namespace wasm_c_api
} // namespace runtimes
} // namespace webrogue
