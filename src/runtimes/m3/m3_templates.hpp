#pragma once

#include "../../../external/wasm3/source/wasm3.h"
#include "../../common/tuple_utils.hpp"
#include "../../core/ModsRuntime.hpp"
#include "../../core/ObjectExtractor.hpp"
#include <tuple>

namespace webrogue {
namespace runtimes {
namespace m3 {

template <char signature> struct TypeToSignatureImpl {
    static const char value = signature;
};
template <typename T, typename = void> struct TypeToSignature;

template <> struct TypeToSignature<void> : TypeToSignatureImpl<'v'> {};
template <>
struct TypeToSignature<core::WASMRawI32> : TypeToSignatureImpl<'i'> {};
template <>
struct TypeToSignature<core::WASMRawU32> : TypeToSignatureImpl<'i'> {};
template <>
struct TypeToSignature<core::WASMRawI64> : TypeToSignatureImpl<'I'> {};
template <>
struct TypeToSignature<core::WASMRawU64> : TypeToSignatureImpl<'I'> {};
template <> struct TypeToSignature<float> : TypeToSignatureImpl<'f'> {};
template <>
struct TypeToSignature<core::WASMRawF64> : TypeToSignatureImpl<'F'> {};

template <typename Ret, typename... Args> struct SignatureHelper {
    static char *getSignature() {
        static char result[] = {TypeToSignature<Ret>::value, '(',
                                TypeToSignature<Args>::value..., ')', 0};
        return result;
    }
};

template <typename Func, Func func> struct MethodToFunc;
template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct MethodToFunc<Ret (Obj::*)(Args...), method> {
    static inline Ret func(IM3ImportContext m3Context, Args... args) {
        return (core::ObjectExtractor<Obj>::get(
                    (webrogue::core::ModsRuntime *)m3Context->userdata)
                    ->*method)(args...);
    }
};

typedef uint64_t *stack_type;
typedef void *mem_type;

template <typename T>
void argFromStack(T &dest, stack_type &_sp, [[maybe_unused]] mem_type mem) {
    m3ApiGetArg(T, tmp);
    dest = tmp;
}

template <typename T>
void argFromStack(T *&dest, stack_type &_sp, [[maybe_unused]] mem_type _mem) {
    m3ApiGetArgMem(T *, tmp);
    dest = tmp;
};

template <typename T>
void argFromStack(const T *&dest, stack_type &_sp,
                  [[maybe_unused]] mem_type _mem) {
    m3ApiGetArgMem(const T *, tmp);
    dest = tmp;
};

template <typename... Args>
static inline void getArgsFromStack(stack_type &sp, mem_type mem,
                                    std::tuple<Args...> &tuple) {
    tupleForEach(tuple, [&](auto &item) {
        argFromStack(item, sp, mem);
    });
}

template <typename Ret, typename Func, typename ArgsTuple> struct ReturnHelper {
    inline static const void *getReturnValue(Func func, ArgsTuple args,
                                             void *returnPtr) {
        Ret ret = call(func, args);
        Ret *raw_return = (Ret *)returnPtr;
        m3ApiReturn(ret);
    }

    inline static void *getReturnPtr(stack_type &_sp) {
        m3ApiReturnType(Ret);
        return raw_return;
    }
};

template <typename Func, typename ArgsTuple>
struct ReturnHelper<void, Func, ArgsTuple> {
    inline static const void *getReturnValue(Func func, ArgsTuple args,
                                             void *returnPtr) {
        call(func, args);
        m3ApiSuccess();
    }

    inline static void *getReturnPtr(stack_type &sp) {
        return nullptr;
    }
};

template <typename Func, Func func> struct FuncWrapper;
template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct FuncWrapper<Ret (Obj::*)(Args...), method> {
    inline static const void *wrappedFunc([[maybe_unused]] IM3Runtime rt,
                                          IM3ImportContext ctx, stack_type sp,
                                          mem_type mem) {
        std::tuple<Args...> args;

        constexpr auto f = MethodToFunc<Ret (Obj::*)(Args...), method>::func;

        void *rawReturn = ReturnHelper<
            Ret, Ret (*)(IM3ImportContext, Args...),
            std::tuple<IM3ImportContext, Args...>>::getReturnPtr(sp);
        getArgsFromStack(sp, mem, args);
        std::tuple<IM3ImportContext, Args...> nargs =
            std::tuple_cat(std::make_tuple(ctx), args);
        return ReturnHelper<
            Ret, Ret (*)(IM3ImportContext, Args...),
            std::tuple<IM3ImportContext, Args...>>::getReturnValue(f, nargs,
                                                                   rawReturn);
    }
};

template <typename Func, Func func> struct FuncLinker;

template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct FuncLinker<Ret (Obj::*)(Args...), method> {
    inline static M3Result link(std::string moduleName,
                                std::string functionName, IM3Module module,
                                core::ModsRuntime *runtime) {
        auto wrappedFunc =
            FuncWrapper<Ret (Obj::*)(Args...), method>::wrappedFunc;
        return m3_LinkRawFunctionEx(
            module, moduleName.c_str(), functionName.c_str(),
            SignatureHelper<Ret, Args...>::getSignature(), wrappedFunc,
            (void *)(runtime));
    }
};

} // namespace m3
} // namespace runtimes
} // namespace webrogue
