#include "emscripten.h"
#include "emscripten/threading.h"
#include <errno.h>
#include <pthread.h>
#include <stdint.h>

// clang-format off
EM_ASYNC_JS(void, _wr_exec_func, (const char *funcNamePtr), {
    await WebAssembly.promising(
        Module.wrInstance.exports[UTF8ToString(funcNamePtr)]
    )();
});
EM_ASYNC_JS(void, _wr_exec_func_ii, (const char *funcNamePtr, int32_t arg0, int32_t arg1), {
  await WebAssembly.promising(
      Module.wrInstance.exports[UTF8ToString(funcNamePtr)]
  )(arg0, arg1);
});
EM_JS(void, _wr_init_wasm_module, (const uint8_t *pointer, int size), {
    let modsWasmData = HEAPU8.subarray(pointer, pointer + size);
    Module.wrModule = new WebAssembly.Module(modsWasmData);
});

EM_JS(void, _wr_init_wasm_instance, (void *context, const char *jsonPtr), {
    let namesJson = UTF8ToString(jsonPtr);
    let importFuncNames = JSON.parse(namesJson);
    let importObject = {};

    for (const [importModuleName, importedFuncs] of Object.entries(importFuncNames)) {
        let importModule = {};
        for (const [funcName, funcDetails] of Object.entries(importedFuncs)) {
            const retType = funcDetails.ret_type;
            const funcId = funcDetails.func_id;
            var func = undefined;
            if(funcName == "present" || funcName == "poll_oneoff" ) {
                func = new WebAssembly.Suspending(async function (...args) {
                    Module.wrArgs = args;
                    await Module._wr_rs_exported_async_fn(funcId, context);
                    return Module.wrResult;
                });
            } else {
                func = function (...args) {
                    Module.wrArgs = args;
                    Module._wr_rs_exported_fn(funcId, context);
                    return Module.wrResult;
                };
            }
            importModule[funcName] = func;
        }
        importObject[importModuleName] = importModule;
    }
    if(Module.wrSharedMemory) {
        if(!importObject["env"]) {
            importObject["env"] = {};
        }
        importObject["env"]["memory"] = Module.wrSharedMemory;
    }

    Module.wrInstance = new WebAssembly.Instance(Module.wrModule, importObject);
    Module.wrGetMemory = function () { return Module.wrInstance.exports.memory.buffer };
});

EM_ASYNC_JS(void, _wr_rs_thread_wait, (), {
  while(!Module.wrModule) {
    await new Promise(resolve => setTimeout(resolve, 100));
  }
});

extern void wr_init_wasm_module(void *context, const char *jsonPtr,
                                const uint8_t *pointer, uint32_t size) {
  _wr_init_wasm_module(pointer, size);
  _wr_init_wasm_instance(context, jsonPtr);
}
extern void wr_reset_wasm() {
  EM_ASM({
    delete Module.wrSharedMemory;
    delete Module.wrSharedMemoryBuffer;
  });
}
extern void wr_exec_func(const char *funcNamePtr) {
  _wr_exec_func(funcNamePtr);
}
extern void wr_exec_func_ii(const char *funcNamePtr, int32_t arg0,
                            int32_t arg1) {
  _wr_exec_func_ii(funcNamePtr, arg0, arg1);
}
extern uint32_t wr_error_size() {
  return EM_ASM_INT({ 
    return Module.wrModError ? Module.wrModError.length : 0; 
  });
}
extern void wr_error_data(char *error) {
  EM_ASM({
    HEAP8.set(Module.wrModError, $0);
  }, error);
}

#define ARG_GET(T)                                                             \
  EM_JS(T, _wr_get_arg_##T, (uint32_t argNum),                                 \
        { return Module.wrArgs[argNum]; });                                    \
  extern T wr_get_arg_##T(uint32_t argNum) { return _wr_get_arg_##T(argNum); }

ARG_GET(int32_t)
ARG_GET(int64_t)
ARG_GET(uint32_t)
ARG_GET(uint64_t)
ARG_GET(float)
ARG_GET(double)

#define RET_SET(T)                                                             \
  EM_JS(void, _wr_set_ret_##T, (T result), { Module.wrResult = result; });     \
  extern void wr_set_ret_##T(T result) { _wr_set_ret_##T(result); }

RET_SET(int32_t)
RET_SET(int64_t)
RET_SET(uint32_t)
RET_SET(uint64_t)
RET_SET(float)
RET_SET(double)

extern void wr_read_memory(uint32_t modPtr, uint32_t size, char *hostPtr) {
  EM_ASM({
    HEAP8.set(new Int8Array(Module.wrGetMemory().slice($0, $0 + $1)), $2);
  },
  modPtr, size, hostPtr);
}
extern void wr_write_memory(uint32_t modPtr, uint32_t size,
                            const char *hostPtr) {
  EM_ASM({
    (new Int8Array(Module.wrGetMemory())).set(new Int8Array(HEAP8.slice($2, $2 + $1)), $0);
  }, modPtr, size, hostPtr);
}
extern void wr_rs_sleep(uint32_t ms) { emscripten_sleep(ms); }

EM_JS(uint32_t, _wr_memory_size, (), { 
  return Module.wrGetMemory().byteLength; 
});
extern uint32_t wr_memory_size() { return _wr_memory_size(); }

extern void wr_make_shared_memory(uint32_t inital_pages, uint32_t max_pages) {
  EM_ASM({
    const memory = new WebAssembly.Memory({
      initial : $0,
      maximum : $1,
      shared : true,
    });
    Module.wrSharedMemory = memory;
    Module.wrSharedMemoryBuffer = memory.buffer;
  },
  inital_pages, max_pages);
}

extern uint64_t wr_thread_start_listening() {
  EM_ASM({
    if(Module.wrModule) {
      delete Module.wrModule;
    }
    let old_onmessage = self.onmessage;
    let new_onmessage = function(e) {
      old_onmessage(e);
      if (e.data.wr_proxy_data) {
        let data = e.data.wr_proxy_data;
        Module.wrSharedMemory = data.wrSharedMemory;
        Module.wrModule = data.wasmModule;
      }
    };
    self.onmessage = new_onmessage;
  });
  return (uint64_t)pthread_self();
}
extern void wr_thread_send_message(uint64_t tid) {
  uint64_t current_tid = (uint64_t)pthread_self();
  MAIN_THREAD_EM_ASM({
    let worker = PThread.pthreads[$0];
    if (!worker.wrIsObserverRegistered) {
      worker.wrIsObserverRegistered = true;
      let old_onmessage = worker.onmessage;
      let new_onmessage = function(e) {
        old_onmessage(e);
        if (e.data.wr_proxy_to_pthread) {
          let t = PThread.pthreads[e.data.wr_proxy_to_pthread];
          t.postMessage({wr_proxy_data : e.data.wr_proxy_data});
        }
      };
      worker.onmessage = new_onmessage;
    }
  }, current_tid);
  EM_ASM({
    self.postMessage({
      wr_proxy_to_pthread : $0,
      wr_proxy_data : {
        wrSharedMemory : Module.wrSharedMemory,
        wasmModule : Module.wrModule
      }
    })
  }, tid);
}
extern void wr_rs_thread_wait(void *context, const char *jsonPtr) {
  _wr_rs_thread_wait();
  _wr_init_wasm_instance(context, jsonPtr);
}


EM_JS(void, wr_reset_timer, (), {
  Module.wr_timer = new Date();
});
EM_JS(uint64_t, wr_get_timer, (), {
  return BigInt(new Date() - Module.wr_timer);
});
