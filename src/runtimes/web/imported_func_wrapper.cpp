#include "imported_func_wrapper.hpp"
#include "../../common/tuple_utils.hpp"
#include "../../core/ApiObject.hpp"
#include "../../core/ModsRuntime.hpp"
#include "../../core/ObjectExtractor.hpp"
#include "../../core/WASIObject.hpp"
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <iostream>
#include <map>
#include <sstream>
#include <string>
#include <vector>

#ifdef __EMSCRIPTEN__
#include "emscripten.h"
#else
#define EM_JS(ret_type, func_name, args, body)                                 \
    ret_type func_name args {                                                  \
        abort();                                                               \
    }
#endif
// clang-format off
EM_JS(void, makeWorker, (const char *jsonPtr), {
        Module.modsWorker = new Worker("worker.js");
        let namesJson = UTF8ToString(jsonPtr);
        Module.importedFuncNames = JSON.parse(namesJson);
        Module.modError = undefined;
    });
EM_JS(void, terminateWorker, (), {
        Module.modsWorker.terminate();
        delete Module.modsWorker;
        Module.executionFinished = true;
    });
EM_JS(int32_t, getArgInt32, (uint32_t argNum), {
        return Module.importedFuncArgs[argNum]
    });
EM_JS(uint32_t, getArgUInt32, (uint32_t argNum), {
        return Module.importedFuncArgs[argNum]
    });
EM_JS(int64_t, getArgInt64, (uint32_t argNum), {
        return Module.importedFuncArgs[argNum]
    });
EM_JS(uint64_t, getArgUInt64, (uint32_t argNum), {
        return Module.importedFuncArgs[argNum]
    });
EM_JS(float, getArgFloat, (uint32_t argNum), {
        return Module.importedFuncArgs[argNum]
    });
EM_JS(double, getArgDouble, (uint32_t argNum), {
        return Module.importedFuncArgs[argNum]
    });
EM_JS(void, writeInt32Result, (int32_t result), {
        Module.workerSharedArray[1] = BigInt(result)
    });
EM_JS(void, writeUInt32Result, (uint32_t result), {
        Module.workerSharedArray[1] = BigInt(result)
    });
EM_JS(void, writeInt64Result, (int64_t result), {
        Module.workerSharedArray[1] = result
    });
EM_JS(void, writeUInt64Result, (uint64_t result), {
        Module.workerSharedArray[1] = result
    });
EM_JS(void, writeFloatResult, (float result), {
        let buffer = new ArrayBuffer(8);
        (new Float64Array(buffer))[0] = result;
        Module.workerSharedArray[1] = (new BigInt64Array(buffer))[0];
    });
EM_JS(void, writeDoubleResult, (double result), {
        let buffer = new ArrayBuffer(8);
        (new Float64Array(buffer))[0] = result;
        Module.workerSharedArray[1] = (new BigInt64Array(buffer))[0];
    });
// clang-format on

namespace webrogue {
namespace runtimes {
namespace web {

void writeResult(core::WASMRawI32 result) {
    writeInt32Result(result.get());
}

void writeResult(core::WASMRawU32 result) {
    writeUInt32Result(result.get());
}

void writeResult(core::WASMRawI64 result) {
    writeInt64Result(result.get());
}

void writeResult(core::WASMRawU64 result) {
    writeUInt64Result(result.get());
}

void writeResult(core::WASMRawF32 result) {
    writeFloatResult(result.get());
}

void writeResult(core::WASMRawF64 result) {
    writeDoubleResult(result.get());
}

void argFromStack(core::WASMRawI32 &dest, size_t argNum) {
    dest = core::WASMRawI32::make(getArgInt32(argNum));
}

void argFromStack(core::WASMRawU32 &dest, size_t argNum) {
    dest = core::WASMRawU32::make(getArgUInt32(argNum));
}

void argFromStack(core::WASMRawI64 &dest, size_t argNum) {
    dest = core::WASMRawI64::make(getArgInt64(argNum));
}

void argFromStack(core::WASMRawU64 &dest, size_t argNum) {
    dest = core::WASMRawU64::make(getArgUInt64(argNum));
}

void argFromStack(core::WASMRawF32 &dest, size_t argNum) {
    dest = core::WASMRawF32::make(getArgFloat(argNum));
}

void argFromStack(core::WASMRawF64 &dest, size_t argNum) {
    dest = core::WASMRawF64::make(getArgDouble(argNum));
}

typedef void (*wrappedFunc)();

template <typename... Args>
inline void getArgsFromStack(std::tuple<Args...> &tuple) {
    size_t argNum = 0;

    tupleForEach(tuple, [&](auto &item) {
        argFromStack(item, argNum++);
    });
}

webrogue::core::ModsRuntime *sharedRuntime = nullptr;

template <typename Func, Func func> struct func_helper;
template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct func_helper<Ret (Obj::*)(Args...), method> {
    static inline Ret fn(Args... args) {
        return (core::ObjectExtractor<Obj>::get(sharedRuntime)->*method)(
            args...);
    }
};

template <typename Ret, typename Func, typename ArgsTuple> struct ReturnHelper {
    inline static void callAndReturn(Func func, ArgsTuple args) {
        Ret ret = call(func, args);

        writeResult(ret);
    }
};

template <typename Func, typename ArgsTuple>
struct ReturnHelper<void, Func, ArgsTuple> {
    inline static void callAndReturn(Func func, ArgsTuple args) {
        call(func, args);
    }
};

inline static void procExitFunc() {
    int const exitCode = getArgUInt32(0);
    sharedRuntime->procExit = true;
    sharedRuntime->procExitCode = exitCode;
    terminateWorker();
}

template <typename Func, Func func> struct FuncWrapper;
template <typename Obj, typename Ret, typename... Args,
          Ret (Obj::*method)(Args...)>
struct FuncWrapper<Ret (Obj::*)(Args...), method> {
    inline static void wrappedFunc() {

        std::tuple<Args...> args;
        getArgsFromStack(args);

        constexpr auto f = func_helper<Ret (Obj::*)(Args...), method>::fn;

        ReturnHelper<Ret, Ret (*)(Args...), std::tuple<Args...>>::callAndReturn(
            f, args);
    }
};
template <typename RetType> struct TypeToSignature;
template <> struct TypeToSignature<void> {
    static std::string getStr() {
        return "void";
    }
};
template <> struct TypeToSignature<core::WASMRawI32> {
    static std::string getStr() {
        return "int32_t";
    }
};
template <> struct TypeToSignature<core::WASMRawU32> {
    static std::string getStr() {
        return "uint32_t";
    }
};
template <> struct TypeToSignature<core::WASMRawI64> {
    static std::string getStr() {
        return "int64_t";
    }
};
template <> struct TypeToSignature<core::WASMRawU64> {
    static std::string getStr() {
        return "uint64_t";
    }
};

template <> struct TypeToSignature<core::WASMRawF32> {
    static std::string getStr() {
        return "float";
    }
};
template <> struct TypeToSignature<core::WASMRawF64> {
    static std::string getStr() {
        return "double";
    }
};

std::vector<wrappedFunc> wrappedFuncs;

#define ADD_TYPE(NAME) typedef core::NAME NAME
ADD_TYPE(WASMRawI32);
ADD_TYPE(WASMRawU32);
ADD_TYPE(WASMRawI64);
ADD_TYPE(WASMRawU64);
ADD_TYPE(WASMRawF32);
ADD_TYPE(WASMRawF64);

void initWrapper(webrogue::core::ModsRuntime *runtime) {
    sharedRuntime = runtime;

    std::stringstream json;
    bool needsComma = false;
#define JSON_ENTRY(NAME, RET_TYPE)                                             \
    json << (needsComma ? "," : "") << "\"" #NAME "\":{\"ret_type\":\""        \
         << TypeToSignature<RET_TYPE>::getStr()                                \
         << "\",\"func_id\":" << wrappedFuncs.size() << "}";                   \
    needsComma = true

    needsComma = false;
    json << "{\"webrogue\": {";
#define WR_API_FUNCTION(RET_TYPE, NAME, ARGS)                                  \
    JSON_ENTRY(NAME, RET_TYPE);                                                \
    wrappedFuncs.push_back(FuncWrapper<decltype(&core::ApiObject::NAME),       \
                                       &core::ApiObject::NAME>::wrappedFunc);

#include "../../../mods/core/include/common/wr_api_functions.def"
#undef WR_API_FUNCTION

    needsComma = false;
    json << "},\"wasi_snapshot_preview1\": {";
#define WASI_FUNCTION(RET_TYPE, NAME, ARGS)                                    \
    JSON_ENTRY(NAME, RET_TYPE);                                                \
    wrappedFuncs.push_back(FuncWrapper<decltype(&core::WASIObject::NAME),      \
                                       &core::WASIObject::NAME>::wrappedFunc);

#include "../../core/wasi_functions.def"
#undef WASI_FUNCTION
    {
        JSON_ENTRY(proc_exit, void);
        wrappedFuncs.push_back(procExitFunc);
    }
    json << "}}";

#undef JSON_ENTRY

    std::string const jsonStr = json.str();

    makeWorker(jsonStr.c_str());
}

void callImportedFunc(int funcId) {
    wrappedFuncs[funcId]();
}
} // namespace web
} // namespace runtimes
} // namespace webrogue
