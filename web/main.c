#include "emscripten.h"
#include <emscripten/wasmfs.h>
#include <stdint.h>

extern void rust_main();

int main(int argc, const char **argv) {
  wasmfs_create_memory_backend();
  rust_main();
}
