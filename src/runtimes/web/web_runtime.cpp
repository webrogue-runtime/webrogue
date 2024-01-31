#include "web_runtime.hpp"

#ifdef __EMSCRIPTEN__
#include "emscripten.h"
#else
#define EM_JS(ret_type, func_name, args, body)                                 \
    ret_type func_name args {                                                  \
        abort();                                                               \
    }
#endif
#include "../../core/CompactLinking.hpp"
#include "imported_func_wrapper.hpp"
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <map>
#include <math.h>
#include <memory>
#include <string>
#include <unistd.h>
#include <vector>

// clang-format off
EM_JS(void, execFunc, (const char *funcNamePtr), {
    Asyncify.handleSleep(wakeUp => {
        Module.modsExecFinished = wakeUp;
        let funcName = UTF8ToString(funcNamePtr);
        Module.modsWorker.postMessage([ "exec", funcName ]);
    });
});
EM_JS(void, readModMem, (uint32_t modPtr, uint32_t size, void *hostPtr), {
    Asyncify.handleSleep(wakeUp => {
        Module.gotMemorySlice = function (memorySlice) {
            HEAP8.set(new Int8Array(memorySlice), hostPtr);
            wakeUp();
        };
        Module.workerSharedArray[1] = BigInt(modPtr);
        Module.workerSharedArray[2] = BigInt(size);
        Atomics.store(Module.workerSharedArray, 0, BigInt(2));
        Atomics.notify(Module.workerSharedArray, 0);
    });
});
EM_JS(void, writeModMem, (uint32_t modPtr, uint32_t size, const void *hostPtr), {
    Asyncify.handleSleep(wakeUp => {
        let dataToWrite = HEAP8.slice(hostPtr, hostPtr + size);
        Module.gotMemorySlice = function (memorySlice) {
            new Int8Array(memorySlice).set(dataToWrite);
            Module.sliceWrote = wakeUp;
            Atomics.store(Module.workerSharedArray, 0, BigInt(3));
            Atomics.notify(Module.workerSharedArray, 0);
        };
        Module.workerSharedArray[1] = BigInt(modPtr);
        Module.workerSharedArray[2] = BigInt(size);
        Atomics.store(Module.workerSharedArray, 0, BigInt(2));
        Atomics.notify(Module.workerSharedArray, 0);
    });
});
EM_JS(void, initWasmModule, (const uint8_t *pointer, int size), {
    Asyncify.handleSleep(wakeUp => {
        Module.workerSharedBuffer = new SharedArrayBuffer(256);
        Module.workerSharedArray = new BigInt64Array(Module.workerSharedBuffer);
        var modsWasmData = HEAPU8.subarray(pointer, pointer + size);

        Module.modsWorker.onmessage = function (message) {
            let command = message.data[0];
            if (command == 0) { // instantiated
                wakeUp();
            } else if (command == 1) { // exec_finished
                Module.executionFinished = true;
        
                let modsExecFinished = Module.modsExecFinished;
                Module.modsExecFinished = undefined;
                modsExecFinished();
            } else if (command == 2) { // exec_imported
                Module.importedFuncId = message.data[1];
                Module.importedFuncArgs = message.data[2];
                Module.executionFinished = false;

                let modsExecFinished = Module.modsExecFinished;
                Module.modsExecFinished = undefined;
                modsExecFinished();
            } else if (command == 3) { // memory_slice
                Module.gotMemorySlice(message.data[1]);
            } else if (command == 4) { // memory_slice_wrote
                Module.sliceWrote();
            } else if (command = 5) { // error
                Module.modError = message.data[1];
                Module.modError = Int8Array.from(Array.from(Module.modError).map(letter => letter.charCodeAt(0)));
                // Module.modErrorStack = message.data[2];
                Module.executionFinished = true;
                
                let modsExecFinished = Module.modsExecFinished;
                Module.modsExecFinished = undefined;
                modsExecFinished();
            } else {
                console.error("host: unknown command ", command);
            }
        }
        Module.modsWorker.postMessage(["instantiate", modsWasmData, Module.importedFuncNames, Module.workerSharedBuffer]);
    });
});
EM_JS(void, continueFuncExecution, (), {
    Asyncify.handleSleep(wakeUp => {
        Module.modsExecFinished = wakeUp;
        Atomics.store(Module.workerSharedArray, 0, BigInt(1));
        Atomics.notify(Module.workerSharedArray, 0);
    });
});
EM_JS(bool, isExecutionFinished, (), {
    return Module.executionFinished;
});
EM_JS(int, getImportedFuncId, (), {
    return Module.importedFuncId;
});
EM_JS(int, modErrorSize, (), {
    return Module.modError ? Module.modError.length : 0
});
EM_JS(int, getModError, (char *error), {
    HEAP8.set(Module.modError, error);
});
// clang-format on

namespace webrogue {
namespace runtimes {
namespace web {

WebModsRuntime::WebModsRuntime(webrogue::core::ConsoleStream *wrout,
                               webrogue::core::ConsoleStream *wrerr,
                               webrogue::core::ResourceStorage *resourceStorage,
                               webrogue::core::Config *config)
    : ModsRuntime(wrout, wrerr, resourceStorage, config){};

void WebModsRuntime::initMods() {
    linkedWasm = getCompactlyLinkedBinaries(
        this, resourceStorage, config,
        [this]() {
            interrupt();
        },
        wrout, wrerr);
    if (!linkedWasm) {
        *wrerr << "linking failed\n";
        return;
    }

    initWrapper(this);

    initWasmModule(linkedWasm->data(), linkedWasm->size());

    execAsyncFunc("__wasm_call_ctors");

    for (std::string const &modName : resourceStorage->modNames) {
        execAsyncFunc("init_mod_" + modName);
    }

    isInitialized = true;
}

bool WebModsRuntime::execAsyncFunc(std::string funcName) {
    execFunc(funcName.c_str());
    while (!isExecutionFinished()) {
        callImportedFunc(getImportedFuncId());
        if (!procExit)
            continueFuncExecution();
    }
    if (int const errorSize = modErrorSize()) {
        std::vector<char> error;
        error.resize(errorSize);
        getModError(error.data());
        *wrerr << std::string(error.data(), error.size()) << "\n";
        return false;
    }
    return true;
}

void WebModsRuntime::start() {
    execAsyncFunc("wr_start");
}
bool WebModsRuntime::getVMData(void *outPtr, uint64_t offset, int32_t size) {
    readModMem(offset, size, outPtr);
    return true;
}
bool WebModsRuntime::setVMData(const void *inPtr, uint64_t offset,
                               int32_t size) {
    writeModMem(offset, size, inPtr);
    return true;
}

size_t WebModsRuntime::voidptrSize() {
    return 4;
};
WebModsRuntime::~WebModsRuntime() {
}
} // namespace web
} // namespace runtimes
} // namespace webrogue
