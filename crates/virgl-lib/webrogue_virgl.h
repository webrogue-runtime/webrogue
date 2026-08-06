#include "virglrenderer.h"
#include "vtest.h"


void webrogueSetVulkan(void* vulkan);

uint8_t webrogue_virgl_is_impl();
void webrogue_virgl_stub_fn();

/* Returns a host pointer to the storage of the venus blob resource identified
 * by blob_id (the vtest res_id shared with the guest), or NULL when unknown.
 */
void *webrogue_get_host_blob(uint64_t blob_id);
