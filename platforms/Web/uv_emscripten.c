#include "emscripten.h"
#include "uv.h"
#include <stdio.h>
#include <stdlib.h>

int uv__platform_loop_init(uv_loop_t *loop) {
    return 0;
}

void uv__platform_loop_delete(uv_loop_t *loop) {
}

void uv__fs_event_close(uv_fs_event_t *handle) {
    EM_ASM({ alert("uv__fs_event_close"); });
}
void uv__io_poll(uv_loop_t *loop, int timeout) {
    EM_ASM({ alert("uv__io_poll"); });
}
void uv__platform_invalidate_fd(uv_loop_t *loop, int fd) {
    EM_ASM({ alert("uv__platform_invalidate_fd"); });
}