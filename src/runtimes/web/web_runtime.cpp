#include "web_runtime.hpp"

#include "../../common/stringize.hpp"
#include "../../core/ApiObject.hpp"
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

namespace webrogue {
namespace runtimes {
namespace web {

WebModsRuntime::WebModsRuntime(webrogue::core::ConsoleStream *wrout,
                               webrogue::core::ConsoleStream *wrerr,
                               webrogue::core::ResourceStorage *resourceStorage,
                               webrogue::core::Config *config)
    : ModsRuntime(wrout, wrerr, resourceStorage, config){};

extern "C" {
extern void readModMem(uint32_t modPtr, uint32_t size, void *hostPtr);
extern void writeModMem(uint32_t modPtr, uint32_t size, const void *hostPtr);
extern void initWasmModule(const uint8_t *mys, int siz);
extern void execFunc(const char *funcName);
extern void continueFuncExecution();
extern bool isExecutionFinished();
extern int getImportedFuncId();
extern int modErrorSize();
extern int getModError(char *error);
}

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

    for (auto modName : resourceStorage->modNames) {
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

#ifndef __EMSCRIPTEN__
extern "C" {
extern void readModMem(uint32_t modPtr, uint32_t size, void *hostPtr) {
    abort();
}
extern void writeModMem(uint32_t modPtr, uint32_t size, const void *hostPtr) {
    abort();
}
extern void initWasmModule(const uint8_t *mys, int siz) {
    abort();
}
extern void execFunc(const char *funcName) {
    abort();
}
extern bool isExecutionFinished() {
    return false;
}
extern int getImportedFuncId() {
    abort();
};

extern int32_t getArgInt32(uint32_t argNum) {
    abort();
}
extern uint32_t getArgUInt32(uint32_t argNum) {
    abort();
}
extern int64_t getArgInt64(uint32_t argNum) {
    if (argNum == 1)
        return 2;
    abort();
}
extern uint64_t getArgUInt64(uint32_t argNum) {
    if (argNum == 0)
        return 1;
    abort();
}
extern float getArgFloat(uint32_t argNum) {
    abort();
}
extern double getArgDouble(uint32_t argNum) {
    abort();
}
extern void writeInt32Result(int32_t result) {
    abort();
}
extern void writeUInt32Result(uint32_t result) {
    abort();
}
extern void writeInt64Result(int64_t result) {
    abort();
}
extern void writeUInt64Result(uint64_t result) {
    abort();
}
extern void writeFloatResult(float result) {
    abort();
}
extern void writeDoubleResult(double result) {
    abort();
}
extern void continueFuncExecution() {
    abort();
}
extern void makeWorker(const char *jsonPtr) {
    std::cout << jsonPtr << "\n";
    abort();
}
extern void terminateWorker() {
    abort();
}
extern int modErrorSize() {
    abort();
}
extern int getModError(char *error) {
    abort();
}
}
#endif
