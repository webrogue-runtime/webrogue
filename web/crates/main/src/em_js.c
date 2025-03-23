#include "emscripten.h"
#include "emscripten/threading.h"
#include <errno.h>
#include <pthread.h>
#include <stdint.h>

// clang-format off
EM_ASYNC_JS(void, wr_em_js_execFunc, (const char *funcNamePtr), {
    await WebAssembly.promising(
        Module.wasmInstance.instance.exports[UTF8ToString(funcNamePtr)]
    )();
});
EM_JS(void, wr_em_js_readModMem, (uint32_t modPtr, uint32_t size, void *hostPtr), {
    HEAP8.set(new Int8Array(Module.getMemory().slice(modPtr, modPtr + size)), hostPtr);
});
EM_JS(void, wr_em_js_writeModMem, (uint32_t modPtr, uint32_t size, const void *hostPtr), {
    (new Int8Array(Module.getMemory())).set(new Int8Array(HEAP8.slice(hostPtr, hostPtr + size)), modPtr);
});
EM_ASYNC_JS(void, wr_em_js_initWasmModule, (void* context, const char *jsonPtr, const uint8_t *pointer, int size), {
    let namesJson = UTF8ToString(jsonPtr);
    let importFuncNames = JSON.parse(namesJson);
    let modsWasmData = HEAPU8.subarray(pointer, pointer + size);
    let importObject = {};

    for (const [importModuleName, importedFuncs] of Object.entries(importFuncNames)) {
        let importModule = {};
        for (const [funcName, funcDetails] of Object.entries(importedFuncs)) {
            const retType = funcDetails.ret_type;
            const funcId = funcDetails.func_id;
            var func = undefined;
            if(funcName == "present" || funcName == "poll_oneoff" ) {
                func = new WebAssembly.Suspending(async function (...args) {
                    Module.wasmArgs = args;
                    await Module._wr_rs_exported_async_fn(funcId, context);
                    return Module.wasmResult;
                });
            } else {
                func = function (...args) {
                    Module.wasmArgs = args;
                    Module._wr_rs_exported_fn(funcId, context);
                    return Module.wasmResult;
                };
            }
            importModule[funcName] = func;
        }
        importObject[importModuleName] = importModule;
    }
    if(Module.sharedMemory) {
        if(!importObject["env"]) {
            importObject["env"] = {};
        }
        importObject["env"]["memory"] = Module.sharedMemory;
    }

    Module.wasmInstance = await WebAssembly.instantiate(modsWasmData, importObject);
    Module.getMemory = function () { return Module.wasmInstance.instance.exports.memory.buffer };
});
EM_JS(int,wr_em_js_modErrorSize, (), {
    return Module.modError ? Module.modError.length : 0;
});
EM_JS(void, wr_em_js_getModError, (char *error), {
    HEAP8.set(Module.modError, error);
});
EM_JS(void, wr_em_js_resetWasm, (), {
    delete Module.sharedMemory;
    delete Module.sharedMemoryBuffer;
});
EM_JS(int32_t, wr_em_js_getArgInt32, (uint32_t argNum), {
    return Module.wasmArgs[argNum];
});
EM_JS(uint32_t, wr_em_js_getArgUInt32, (uint32_t argNum), {
    return Module.wasmArgs[argNum];
});
EM_JS(int64_t, wr_em_js_getArgInt64, (uint32_t argNum), {
    return Module.wasmArgs[argNum];
});
EM_JS(uint64_t, wr_em_js_getArgUInt64, (uint32_t argNum), {
    return Module.wasmArgs[argNum];
});
EM_JS(float, wr_em_js_getArgFloat, (uint32_t argNum), {
    return Module.wasmArgs[argNum];
});
EM_JS(double, wr_em_js_getArgDouble, (uint32_t argNum), {
    return Module.wasmArgs[argNum];
});
EM_JS(void, wr_em_js_writeInt32Result, (int32_t result), {
    Module.wasmResult = result;
});
EM_JS(void, wr_em_js_writeUInt32Result, (uint32_t result), {
    Module.wasmResult = result;
});
EM_JS(void, wr_em_js_writeInt64Result, (int64_t result), {
    Module.wasmResult = result;
});
EM_JS(void, wr_em_js_writeUInt64Result, (uint64_t result), {
    Module.wasmResult = result;
});
EM_JS(void, wr_em_js_writeFloatResult, (float result), {
    Module.wasmResult = result;
});
EM_JS(void, wr_em_js_writeDoubleResult, (double result), {
    Module.wasmResult = result;
});
EM_JS(uint32_t, wr_em_js_memorySize, (), {
    return Module.getMemory().byteLength;
});
EM_JS(void, wr_em_js_makeSharedMemory, (uint32_t inital_pages, uint32_t max_pages), {
    const memory = new WebAssembly.Memory({
        initial: inital_pages,
        maximum: max_pages,
        shared: true,
    });
    Module.sharedMemory = memory;
    Module.sharedMemoryBuffer = memory.buffer;
});
EM_JS(void, wr_em_js_threadStartListening, (), {
  console.log("listening messages");
  let old_onmessage = self.onmessage;
  console.log(old_onmessage);
  let new_onmessage = (e) => {
    console.log("event:", e);
    old_onmessage(e);
  };
  self.onmessage = new_onmessage;
});
EM_JS(void, wr_em_js_threadSendMessage, (uint64_t tid), {
  // let t = PThread.pthreads[tid];
  console.log("sending message");
  postMessage("abobus");
});

EM_ASYNC_JS(void, wr_em_js_threadWait, (), {
  await new Promise(r => setTimeout(r, 10000));
});

// clang-format on

extern void wr_rs_em_js_initWasmModule(void *context, const char *jsonPtr,
                                       const uint8_t *pointer, uint32_t size) {
  wr_em_js_initWasmModule(context, jsonPtr, pointer, size);
}
extern void wr_rs_em_js_resetWasm() { wr_em_js_resetWasm(); }
extern void wr_rs_em_js_execFunc(const char *funcNamePtr) {
  wr_em_js_execFunc(funcNamePtr);
}
extern uint32_t wr_rs_em_js_modErrorSize() { return wr_em_js_modErrorSize(); }
extern void wr_rs_em_js_getModError(char *error) {
  wr_em_js_getModError(error);
}
extern uint32_t wr_rs_em_js_getArgU32(uint32_t argNum) {
  return wr_em_js_getArgUInt32(argNum);
}
extern void wr_rs_em_js_setResultU32(uint32_t result) {
  wr_em_js_writeUInt32Result(result);
}
extern uint64_t wr_rs_em_js_getArgU64(uint32_t argNum) {
  return wr_em_js_getArgUInt64(argNum);
}
extern void wr_rs_em_js_setResultU64(uint64_t result) {
  wr_em_js_writeUInt64Result(result);
}
extern int32_t wr_rs_em_js_getArgI32(uint32_t argNum) {
  return wr_em_js_getArgInt32(argNum);
}
extern void wr_rs_em_js_setResultI32(int32_t result) {
  wr_em_js_writeInt32Result(result);
}
extern int64_t wr_rs_em_js_getArgI64(uint32_t argNum) {
  return wr_em_js_getArgInt64(argNum);
}
extern void wr_rs_em_js_setResultI64(int64_t result) {
  wr_em_js_writeInt64Result(result);
}
extern float wr_rs_em_js_getArgF32(uint32_t argNum) {
  return wr_em_js_getArgFloat(argNum);
}
extern void wr_rs_em_js_setResultF32(float result) {
  wr_em_js_writeFloatResult(result);
}
extern double wr_rs_em_js_getArgF64(uint32_t argNum) {
  return wr_em_js_getArgDouble(argNum);
}
extern void wr_rs_em_js_setResultF64(double result) {
  wr_em_js_writeDoubleResult(result);
}
extern void wr_rs_em_js_readModMem(uint32_t modPtr, uint32_t size,
                                   char *hostPtr) {
  wr_em_js_readModMem(modPtr, size, hostPtr);
}
extern void wr_rs_em_js_writeModMem(uint32_t modPtr, uint32_t size,
                                    const char *hostPtr) {
  wr_em_js_writeModMem(modPtr, size, hostPtr);
}
extern void wr_rs_sleep(uint32_t ms) { emscripten_sleep(ms); }
extern uint32_t wr_rs_em_js_memorySize() { return wr_em_js_memorySize(); }
extern void wr_rs_em_js_makeSharedMemory(uint32_t inital_pages,
                                         uint32_t max_pages) {
  wr_em_js_makeSharedMemory(inital_pages, max_pages);
}

extern uint64_t wr_rs_threadStartListening() {
  wr_em_js_threadStartListening();
  return (uint64_t)pthread_self();
}
// static void _wr_rs_threadSendMessage(uint64_t *tid) {
//   wr_em_js_threadSendMessage(*tid);
// }
extern void wr_rs_threadSendMessage(uint64_t tid) {
  MAIN_THREAD_EM_ASM(
      {
        let t = PThread.pthreads[$0];
        console.log("sending message to ", t);
        t.postMessage("abobus");
      },
      tid);
}
extern void wr_rs_threadWait() { wr_em_js_threadWait(); }

void *errno_location() { return __errno_location(); }

typedef void (*onMainCallback)(void *userdata);
extern void webrogueRunOnMainThread(onMainCallback f, void *userdata) {
  f(userdata);
  // emscripten_sync_run_in_main_runtime_thread(EM_FUNC_SIG_VI, f, userdata);
}

// emscripten_main_runtime_thread_id

//     emscripten_proxy_sync
