#include "emscripten.h"
#include <emscripten/wasmfs.h>
#include <stdint.h>
#include <stdio.h>

extern void rust_main();

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
  rust_main();
}
