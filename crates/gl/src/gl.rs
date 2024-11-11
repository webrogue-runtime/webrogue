use std::sync::{Arc, Mutex};

struct GLOpaqueObjects {
    pub opaque_sync_objects: std::collections::BTreeMap<u32, *mut ()>,
    pub opaque_sync_object_counter: u32,
}

impl GLOpaqueObjects {
    fn new() -> Self {
        Self {
            opaque_sync_objects: std::collections::BTreeMap::new(),
            opaque_sync_object_counter: 1,
        }
    }

    pub fn register_opaque_sync_object(&mut self, object: *mut ()) -> u32 {
        let result = self.opaque_sync_object_counter;
        self.opaque_sync_objects
            .insert(self.opaque_sync_object_counter, object);
        self.opaque_sync_object_counter += 1;
        return result;
    }

    pub fn resolve_opaque_sync_object(&mut self, handle: u32) -> *mut () {
        self.opaque_sync_objects
            .get(&handle)
            .unwrap_or(&std::ptr::null_mut())
            .clone()
    }

    pub fn delete_opaque_sync_object(&mut self, handle: u32) {
        self.opaque_sync_objects.remove(&handle);
    }
}

pub struct GL {
    pub gfx: Arc<webrogue_gfx::GFX>,
    pub proc_addresses: crate::proc_addresses::ProcAddresses,
    opaque_objects: Arc<Mutex<GLOpaqueObjects>>,
}

impl GL {
    pub fn new(gfx: Arc<webrogue_gfx::GFX>) -> Self {
        Self {
            gfx,
            proc_addresses: crate::proc_addresses::ProcAddresses::new(),
            opaque_objects: Arc::new(Mutex::new(GLOpaqueObjects::new())),
        }
    }

    pub fn register_opaque_sync_object(&self, object: *mut ()) -> u32 {
        self.opaque_objects
            .lock()
            .unwrap()
            .register_opaque_sync_object(object)
    }

    pub fn resolve_opaque_sync_object(&self, handle: u32) -> *mut () {
        self.opaque_objects
            .lock()
            .unwrap()
            .resolve_opaque_sync_object(handle)
    }

    pub fn delete_opaque_sync_object(&self, handle: u32) {
        self.opaque_objects
            .lock()
            .unwrap()
            .delete_opaque_sync_object(handle)
    }
}
