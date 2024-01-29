#pragma once

#include "../../common/tuple_utils.hpp"
#include "../../core/ModsRuntime.hpp"
#include "../../core/ObjectExtractor.hpp"
#include "wasmedge/wasmedge.h"
#include <cstdint>
#include <string>

namespace webrogue {
namespace runtimes {
namespace wasmedge {

typedef WasmEdge_ValType TypeFactory(void);

template <TypeFactory signature> struct TypeToSignatureImpl {
    static inline WasmEdge_ValType genType() {
        return signature();
    }
};
template <typename T, typename = void> struct TypeToSignature;

template <>
struct TypeToSignature<core::WASMRawI32>
    : TypeToSignatureImpl<WasmEdge_ValTypeGenI32> {};
template <>
struct TypeToSignature<core::WASMRawU32>
    : TypeToSignatureImpl<WasmEdge_ValTypeGenI32> {};
template <>
struct TypeToSignature<core::WASMRawI64>
    : TypeToSignatureImpl<WasmEdge_ValTypeGenI64> {};
template <>
struct TypeToSignature<core::WASMRawU64>
    : TypeToSignatureImpl<WasmEdge_ValTypeGenI64> {};
template <>
struct TypeToSignature<core::WASMRawF32>
    : TypeToSignatureImpl<WasmEdge_ValTypeGenF32> {};
template <>
struct TypeToSignature<core::WASMRawF64>
    : TypeToSignatureImpl<WasmEdge_ValTypeGenF64> {};

template <typename... Args> struct ArgumentsSignatureHelper {
    static WasmEdge_ValType *getSignature(size_t &count) {
        static WasmEdge_ValType result[] = {
            TypeToSignature<Args>::genType()...};
        count = sizeof(result) / sizeof(WasmEdge_ValType);
        return result;
    }
};

template <> struct ArgumentsSignatureHelper<> {
    static WasmEdge_ValType *getSignature(size_t &count) {
        count = 0;
        return nullptr;
    }
};

template <typename Ret> struct ReturnSignatureHelper {
    static WasmEdge_ValType *getSignature(size_t &count) {
        static WasmEdge_ValType result[] = {TypeToSignature<Ret>::genType()};
        count = 1;
        return result;
    }
};

template <> struct ReturnSignatureHelper<void> {
    static WasmEdge_ValType *getSignature(size_t &count) {
        count = 0;
        return nullptr;
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

template <typename T> struct ArgumentsStackHelper;

template <> struct ArgumentsStackHelper<core::WASMRawI32> {
    static inline void
    argFromStack(core::WASMRawI32 &dest, const WasmEdge_Value *in, size_t argI,
                 const WasmEdge_CallingFrameContext *callFrameCxt) {
        dest = core::WASMRawI32::make(WasmEdge_ValueGetI32(in[argI]));
    }
};
template <> struct ArgumentsStackHelper<core::WASMRawU32> {
    static inline void
    argFromStack(core::WASMRawU32 &dest, const WasmEdge_Value *in, size_t argI,
                 const WasmEdge_CallingFrameContext *callFrameCxt) {
        dest = core::WASMRawU32::make(WasmEdge_ValueGetI32(in[argI]));
    }
};
template <> struct ArgumentsStackHelper<core::WASMRawI64> {
    static inline void
    argFromStack(core::WASMRawI64 &dest, const WasmEdge_Value *in, size_t argI,
                 const WasmEdge_CallingFrameContext *callFrameCxt) {
        dest = core::WASMRawI64::make(WasmEdge_ValueGetI64(in[argI]));
    }
};
template <> struct ArgumentsStackHelper<core::WASMRawU64> {
    static inline void
    argFromStack(core::WASMRawU64 &dest, const WasmEdge_Value *in, size_t argI,
                 const WasmEdge_CallingFrameContext *callFrameCxt) {
        dest = core::WASMRawU64::make(WasmEdge_ValueGetI64(in[argI]));
    }
};
template <> struct ArgumentsStackHelper<core::WASMRawF32> {
    static inline void
    argFromStack(core::WASMRawF32 &dest, const WasmEdge_Value *in, size_t argI,
                 const WasmEdge_CallingFrameContext *callFrameCxt) {
        dest = core::WASMRawF32::make(WasmEdge_ValueGetF32(in[argI]));
    }
};
template <> struct ArgumentsStackHelper<core::WASMRawF64> {
    static inline void
    argFromStack(core::WASMRawF64 &dest, const WasmEdge_Value *in, size_t argI,
                 const WasmEdge_CallingFrameContext *callFrameCxt) {
        dest = core::WASMRawF64::make(WasmEdge_ValueGetF64(in[argI]));
    }
};
template <> struct ArgumentsStackHelper<void *> {
    static inline void
    argFromStack(void *&dest, const WasmEdge_Value *in, size_t argI,
                 const WasmEdge_CallingFrameContext *callFrameCxt) {
        WasmEdge_MemoryInstanceContext *memCxt =
            WasmEdge_CallingFrameGetMemoryInstance(callFrameCxt, 0);
        size_t const memSize = WasmEdge_MemoryInstanceGetPageSize(memCxt);
        void *mem = WasmEdge_MemoryInstanceGetPointer(memCxt, 0, memSize);
        dest =
            static_cast<void *>(((char *)mem) + WasmEdge_ValueGetI32(in[argI]));
    }
};

template <typename T>
void argFromStack(T &dest, const WasmEdge_Value *in, size_t argI,
                  const WasmEdge_CallingFrameContext *callFrameCxt) {
    ArgumentsStackHelper<T>::argFromStack(dest, in, argI, callFrameCxt);
}

template <typename... Args>
static inline void
getArgsFromStack(const WasmEdge_Value *in,
                 const WasmEdge_CallingFrameContext *callFrameCxt,
                 std::tuple<Args...> &tuple) {
    size_t argI = 0;

    tupleForEach(tuple, [&](auto &item) {
        argFromStack(item, in, argI, callFrameCxt);
        argI++;
    });
}

template <typename Ret, typename Func, typename ArgsTuple> struct ReturnHelper;
template <typename Func, typename ArgsTuple>
struct ReturnHelper<void, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     WasmEdge_Value *out) {
        call(func, args);
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawI32, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     WasmEdge_Value *out) {
        out[0] = WasmEdge_ValueGenI32(call(func, args).get());
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawU32, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     WasmEdge_Value *out) {
        out[0] = WasmEdge_ValueGenI32(call(func, args).get());
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawI64, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     WasmEdge_Value *out) {
        out[0] = WasmEdge_ValueGenI64(call(func, args).get());
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawU64, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     WasmEdge_Value *out) {
        out[0] = WasmEdge_ValueGenI64(call(func, args).get());
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawF32, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     WasmEdge_Value *out) {
        out[0] = WasmEdge_ValueGenF32(call(func, args).get());
    }
};
template <typename Func, typename ArgsTuple>
struct ReturnHelper<core::WASMRawF64, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args,
                                     WasmEdge_Value *out) {
        out[0] = WasmEdge_ValueGenF64(call(func, args).get());
    }
};

template <typename Func, Func func> struct PrimaryFuncWrapper;
template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct PrimaryFuncWrapper<Ret (Obj::*)(Args...), method> {
    static inline WasmEdge_Result
    wrappedFunc(void *data, const WasmEdge_CallingFrameContext *callFrameCxt,
                const WasmEdge_Value *in, WasmEdge_Value *out) {
        std::tuple<Args...> args;

        constexpr auto f =
            SecondaryFuncWrapper<Ret (Obj::*)(Args...), method>::wrappedFunc;

        getArgsFromStack(in, callFrameCxt, args);
        std::tuple<void *, Args...> nargs =
            std::tuple_cat(std::make_tuple(data), args);
        ((core::ModsRuntime *)data)->vmContext = callFrameCxt;
        ReturnHelper<Ret, decltype(f),
                     std::tuple<void *, Args...>>::callAndReturn(f, nargs, out);
        return WasmEdge_Result_Success;
    }
};

template <typename Func, Func func> struct FuncLinker;

template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct FuncLinker<Ret (Obj::*)(Args...), method> {
    inline static void link(std::string moduleName, std::string functionName,
                            WasmEdge_ModuleInstanceContext *hostMod,
                            core::ModsRuntime *runtime) {
        constexpr auto wrappedFunc =
            &(PrimaryFuncWrapper<Ret (Obj::*)(Args...), method>::wrappedFunc);

        size_t paramCount = 0;
        WasmEdge_ValType *paramList =
            ArgumentsSignatureHelper<Args...>::getSignature(paramCount);

        size_t returnCount = 0;
        WasmEdge_ValType *returnList =
            ReturnSignatureHelper<Ret>::getSignature(returnCount);

        WasmEdge_FunctionTypeContext *hostFType = WasmEdge_FunctionTypeCreate(
            paramList, paramCount, returnList, returnCount);

        WasmEdge_FunctionInstanceContext *hostFunc =
            WasmEdge_FunctionInstanceCreate(hostFType, wrappedFunc, runtime, 0);

        WasmEdge_FunctionTypeDelete(hostFType);

        WasmEdge_String const hostName =
            WasmEdge_StringCreateByCString(functionName.c_str());
        WasmEdge_ModuleInstanceAddFunction(hostMod, hostName, hostFunc);
        WasmEdge_StringDelete(hostName);
    }
};
} // namespace wasmedge
} // namespace runtimes
} // namespace webrogue
