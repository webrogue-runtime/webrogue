use ash::Entry;

pub fn load() -> Option<Entry> {
    unsafe {
        let path = std::env::current_exe()
            .ok()?
            .parent()?
            .join("vk_swiftshader.dll");
        Entry::load_from(path).ok()
    }
}
