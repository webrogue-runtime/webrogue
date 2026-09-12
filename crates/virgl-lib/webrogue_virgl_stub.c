#include "webrogue_virgl.h"
#include <stdlib.h>

uint8_t webrogue_virgl_is_impl() {
  return 0;
}

void webrogue_virgl_stub_fn() {}


#define STUB_BODY { abort(); }

bool webrogue_vkr_init(uint32_t flags,
                       void (*retire_fence)(uint32_t ctx_id, uint32_t ring_idx,
                                            uint64_t fence_id)) STUB_BODY
void *webrogue_get_host_blob(uint64_t blob_id) STUB_BODY
void webrogue_virgl_setup_shmem(void *ptr, size_t size) STUB_BODY
void webrogueSetVulkan(void *vulkan) STUB_BODY