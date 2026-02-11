#include <stdlib.h>
#include <cstdint>

extern "C" {
void webrogue_gfxstream_ffi_create_global_state(void *get_proc, void* userdata) {
  abort();
}

void webrogue_gfxstream_ffi_destroy_global_state() {
  abort();
}

void* webrogue_gfxstream_ffi_create_decoder() {
  abort();
}

void webrogue_gfxstream_ffi_destroy_decoder(void *raw_decoder_ptr) {
  abort();
}

void webrogue_gfxstream_ffi_commit_buffer(void *raw_decoder_ptr, void const* buf, uint32_t len) {
  abort();
}

void webrogue_gfxstream_ffi_ret_buffer_read(void *raw_decoder_ptr, void* buf, uint32_t len) {
  abort();
}
void* webrogue_gfxstream_ffi_unbox_vk_instance(uint64_t vk_instance) {
  abort();
}
uint64_t webrogue_gfxstream_ffi_box_vk_surface(void *vk_surface) {
  abort();
}
void webrogue_gfxstream_ffi_register_blob(
  void* raw_decoder_ptr,
  void* buf,
  uint64_t size,
  uint64_t id
) {
  abort();
}
void webrogue_gfxstream_ffi_set_extensions(
  void* raw_decoder_ptr,
  char** raw_extensions,
  uint32_t count
) {
  abort();
}
void webrogue_gfxstream_ffi_set_presentation_callback(
  void* raw_decoder_ptr,
  void (*callback)(void*),
  void* userdata
) {
  abort();
}
void webrogue_gfxstream_ffi_shadow_blob_copy(
  uint64_t blob_id,
  void* data,
  uint64_t blob_offset,
  uint64_t size,
  uint32_t direction
) {
  abort();
}

void webrogue_gfxstream_ffi_set_register_shadow_blob_callback(
  void (*callback)(void*, uint64_t, uint64_t)
) {
  abort();
}

void webrogue_gfxstream_stub_fn() {}
}
