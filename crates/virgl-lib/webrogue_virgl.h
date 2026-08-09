#include "virglrenderer.h"
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