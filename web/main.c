#include "emscripten.h"
#include <emscripten/wasmfs.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

extern void rust_main();
extern void rust_main_slice(uint32_t size, void *data);

static uint32_t appSize;
static void *appData;

uint64_t wr_allocApp(uint32_t size) {
  appSize = size;
  appData = malloc(size);
  return (uint64_t)appData;
}

#if WEBROGUE_DYNAMIC
// clang-format off
EM_ASYNC_JS(uint32_t, wr_dynamic_load, (), { 
  var transaction = homepageIndexedDB.transaction("apps", 'readonly');
  var allRecords = transaction.objectStore("apps").getAll();
  allRecords.onsuccess = function () {
      Module.modsToLoad = allRecords.result;
      async function resolveBlobs() {
          for (const key in Module.modsToLoad) {
              if (Object.hasOwnProperty.call(Module.modsToLoad, key)) {
                  const modToLoad = Module.modsToLoad[key];
                  modToLoad.data = new Uint8Array(await modToLoad.blob.arrayBuffer());
              }
          }
      }
      resolveBlobs().then(wakeUp);
  };
  return Module.wrGetMemory().byteLength; 
});
// clang-format on
#endif

int main(int argc, const char **argv) {
#if WEBROGUE_JSPI // opfs does not works with jspi for some reason
  backend_t fs = wasmfs_create_memory_backend();
#else
  backend_t fs = wasmfs_create_opfs_backend();
#endif
  int err = wasmfs_create_directory("/data", 0777, fs);
  if (err) {
    printf("Failed to create /data\n");
  }
#if !WEBROGUE_DYNAMIC
  rust_main();
#else
  rust_main_slice(appSize, appData);
#endif
}
