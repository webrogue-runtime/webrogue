use std::ffi::c_char;

extern "C" {
    pub fn webrogue_gfxstream_ffi_commit_buffer(
        raw_decoder_ptr: *const (),
        buf: *const (),
        len: u32,
    );
    pub fn webrogue_gfxstream_ffi_ret_buffer_read(
        raw_decoder_ptr: *const (),
        buf: *mut (),
        len: u32,
    );
    pub fn webrogue_gfxstream_ffi_create_global_state(get_proc: *const (), userdata: *const ());
    pub fn webrogue_gfxstream_ffi_destroy_global_state();
    pub fn webrogue_gfxstream_ffi_create_decoder() -> *const ();
    pub fn webrogue_gfxstream_ffi_destroy_decoder(raw_decoder_ptr: *const ());
    pub fn webrogue_gfxstream_ffi_unbox_vk_instance(vk_instance: u64) -> *mut ();
    pub fn webrogue_gfxstream_ffi_box_vk_surface(vk_surface: *mut ()) -> u64;
    pub fn webrogue_gfxstream_ffi_register_blob(
        raw_decoder_ptr: *const (),
        buf: *mut (),
        size: u64,
        id: u64,
    );
    pub fn webrogue_gfxstream_ffi_set_extensions(
        raw_decoder_ptr: *const (),
        raw_extensions: *const *const c_char,
        count: u32,
    );
    pub fn webrogue_gfxstream_ffi_set_presentation_callback(
        raw_decoder_ptr: *const (),
        callback: unsafe extern "C" fn(*const ()),
        userdata: *const (),
    );
    pub fn webrogue_gfxstream_ffi_shadow_blob_copy(
        blob_id: u64,
        data: *mut (),
        blob_offset: u64,
        size: u64,
        // 0 means device -> vm, 1 means vm -> device
        direction: u32,
    );
    pub fn webrogue_gfxstream_ffi_set_register_shadow_blob_callback(
        callback: unsafe extern "C" fn(*const (), u64, u64),
    );
}
