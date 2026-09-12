#include "virglrenderer.h"
#include "venus/vkr_renderer.h"
#include "drm-uapi/virtgpu_drm.h"
#include "vrend/vrend_iov.h"
#include <unistd.h>

void webrogueSetVulkan(void* vulkan);
void *webrogueGetVulkan(void);

uint8_t webrogue_virgl_is_impl();
void webrogue_virgl_stub_fn();

void *webrogue_get_host_blob(uint64_t blob_id);

void webrogue_virgl_setup_shmem(void *ptr, size_t size);
void *webrogue_virgl_pop_shmem(size_t *out_size);

bool webrogue_vkr_init(uint32_t flags,
                       void (*retire_fence)(uint32_t ctx_id, uint32_t ring_idx,
                                            uint64_t fence_id));