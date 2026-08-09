#include "webrogue_virgl.h"
#include <stdlib.h>

uint8_t webrogue_virgl_is_impl() {
  return 0;
}

void webrogue_virgl_stub_fn() {}


#define STUB_BODY { abort(); }

void virgl_renderer_fill_caps(uint32_t set, uint32_t version, void *caps) STUB_BODY
void virgl_renderer_poll(void) STUB_BODY
int virgl_renderer_context_create_with_flags(uint32_t ctx_id,
                                             uint32_t ctx_flags,
                                             uint32_t nlen,
                                             const char *name) STUB_BODY
int virgl_renderer_submit_cmd(void *buffer,
                              int ctx_id,
                              int ndw) STUB_BODY
void virgl_renderer_get_cap_set(uint32_t cap_set, uint32_t *max_ver,
                                uint32_t *max_size) STUB_BODY
int virgl_renderer_context_create_fence(uint32_t ctx_id,
                                        uint32_t flags,
                                        uint32_t ring_idx,
                                        uint64_t fence_id) STUB_BODY
void *webrogue_get_host_blob(uint64_t blob_id) STUB_BODY
void virgl_renderer_cleanup(void *cookie) STUB_BODY
void webrogue_virgl_setup_shmem(void *ptr, size_t size) STUB_BODY
int virgl_renderer_resource_create_blob(const struct virgl_renderer_resource_create_blob_args *args) STUB_BODY
void virgl_renderer_resource_unref(uint32_t res_handle) STUB_BODY
void virgl_renderer_ctx_attach_resource(int ctx_id, int res_handle) STUB_BODY
void webrogueSetVulkan(void *vulkan) STUB_BODY
int virgl_renderer_init(void *cookie, int flags, struct virgl_renderer_callbacks *cbs) STUB_BODY
void virgl_renderer_context_destroy(uint32_t handle) STUB_BODY
