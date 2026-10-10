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

size_t vkr_get_capset(void *capset, uint32_t flags) STUB_BODY
void vkr_renderer_fini(void) STUB_BODY
bool vkr_renderer_submit_cmd(uint32_t ctx_id, void *cmd, uint32_t size) STUB_BODY
bool vkr_renderer_submit_fence(uint32_t ctx_id, uint32_t flags, uint64_t ring_idx, uint64_t fence_id) STUB_BODY
bool vkr_renderer_create_context(uint32_t ctx_id, uint32_t ctx_flags, uint32_t nlen, const char *name) STUB_BODY
bool vkr_renderer_create_resource(uint32_t ctx_id, uint32_t res_id, uint64_t blob_id, uint64_t blob_size, uint32_t blob_flags, enum virgl_resource_fd_type *out_fd_type, int *out_res_fd, uint32_t *out_map_info, struct virgl_resource_vulkan_info *out_vulkan_info, void **out_mapped_ptr) STUB_BODY
void vkr_renderer_destroy_resource(uint32_t ctx_id, uint32_t res_id) STUB_BODY
void vkr_renderer_destroy_context(uint32_t ctx_id) STUB_BODY
