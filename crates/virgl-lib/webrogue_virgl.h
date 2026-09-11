#include "virglrenderer.h"
#include "venus/vkr_renderer.h"
#include "drm-uapi/virtgpu_drm.h"
#include "vrend/vrend_iov.h"
#include <unistd.h>

void webrogueSetVulkan(void* vulkan);
void *webrogueGetVulkan(void);

uint8_t webrogue_virgl_is_impl();
void webrogue_virgl_stub_fn();

/* Returns a host pointer to the storage of the venus blob resource identified
 * by blob_id (the vtest res_id shared with the guest), or NULL when unknown.
 */
void *webrogue_get_host_blob(uint64_t blob_id);

/* Pending-shmem slot consumed by vkr_context_create_resource for shmem blobs.
 * The renderer state machine itself lives on the Rust side (see webrogue.rs). */
void webrogue_virgl_setup_shmem(void *ptr, size_t size);
void *webrogue_virgl_pop_shmem(size_t *out_size);

/* Direct-dispatch init of the venus core (bypasses the proxy/socket/render
 * server layers entirely). Wraps vkr_renderer_init so the retire-fence
 * callback is a plain function pointer: vkr_renderer_callbacks embeds the
 * va_list-logging type whose bindgen output differs per platform.
 * `flags` are VKR_RENDERER_* bits; both THREAD_SYNC and ASYNC_FENCE_CB are
 * required. Returns false on failure. */
bool webrogue_vkr_init(uint32_t flags,
                       void (*retire_fence)(uint32_t ctx_id, uint32_t ring_idx,
                                            uint64_t fence_id));