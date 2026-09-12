pub trait SystemProxy: Send + Sync {
    fn vk_create_surface_webrogue(
        &self,
        instance: ash::vk::Instance,
        webrogue_window_id: u32,
        p_allocator: *const ash::vk::AllocationCallbacks<'_>,
        p_surface: *mut ash::vk::SurfaceKHR,
    ) -> ash::vk::Result;
    fn get_required_extensions(&self) -> Vec<Vec<std::ffi::c_char>>;
}
