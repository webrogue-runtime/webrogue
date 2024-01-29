let modsModule
let workerSharedArray
let getMemory

onmessage = function (message) {
    let command = message.data[0]
    if (command === "instantiate") {
        let modsWasmData = message.data[1]
        let importFuncNames = message.data[2]
        workerSharedArray = new BigInt64Array(message.data[3]);
        let importObject = {};
        for (const [importModuleName, importedFuncs] of Object.entries(importFuncNames)) {
            let importModule = {}
            for (const [funcName, funcDetails] of Object.entries(importedFuncs)) {
                const retType = funcDetails.ret_type
                const funcId = funcDetails.func_id;
                importModule[funcName] = function (...args) {
                    Atomics.store(workerSharedArray, 0, BigInt(0))
                    postMessage([2, funcId, args]); // exec_imported
                    let modPtr;
                    let size;
                    let slice;
                    while (true) {
                        Atomics.wait(workerSharedArray, 0, BigInt(0));
                        if (workerSharedArray[0] == 1)
                            break;
                        else if (workerSharedArray[0] == 2) {
                            modPtr = Number(workerSharedArray[1]);
                            size = Number(workerSharedArray[2]);
                            slice = new SharedArrayBuffer(size);
                            (new Uint8Array(slice)).set(new Uint8Array(getMemory().slice(modPtr, modPtr + size)));
                            Atomics.store(workerSharedArray, 0, BigInt(0))
                            postMessage([3, slice]); // memory_slice
                        } else if (workerSharedArray[0] == 3) {
                            new Uint8Array(getMemory()).set(new Uint8Array(slice), modPtr);
                            Atomics.store(workerSharedArray, 0, BigInt(0))
                            postMessage([4, slice]); // memory_slice_wrote
                        } else {
                            console.error("worker: unknown buffer command: ", workerSharedArray[0])
                        }
                    }
                    let result
                    if (retType == "void") {
                        result = undefined
                    } else if (retType == "int32_t" || retType == "uint32_t") {
                        result = Number(workerSharedArray[1])
                    } else if (retType == "int64_t" || retType == "uint64_t") {
                        result = workerSharedArray[1]
                    } else if (retType == "float" || retType == "double") {
                        let buffer = new ArrayBuffer(8);
                        (new BigInt64Array(buffer))[0] = workerSharedArray[1];
                        result = (new Float64Array(buffer))[0];
                    } else {
                        console.error("unknown retType: ", retType)
                    }
                    return result
                }
            }
            importObject[importModuleName] = importModule
        }

        WebAssembly.instantiate(modsWasmData, importObject).then((newModule) => {
            modsModule = newModule;
            getMemory = function () { return modsModule.instance.exports.memory.buffer }
            postMessage([0]); // instantiated
        });
    } else if (command === "exec") {
        let funcName = message.data[1]
        try {
            modsModule.instance.exports[funcName]();
            postMessage([1]) // exec_finished
        } catch (error) {
            postMessage([5, error.toString(), error.stack]) // error 
        }
    } else {
        console.error("worker: unknown command ", command)
    }
}
