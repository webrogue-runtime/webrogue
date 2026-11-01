use std::sync::{Arc, Mutex};

use crate::request_queue::RequestQueue;

pub struct System {
    request_queue: Arc<RequestQueue>,
}

impl System {
    pub fn new(request_queue: Arc<RequestQueue>) -> Self {
        Self { request_queue }
    }
}

impl webrogue_gfx::AbstractSystem for System {
    fn make_window(&self) -> webrogue_gfx::Window {
        self.request_queue
            .execute_sync(|args| {
                let window = args
                    .sdl_video
                    .window("a", 1, 1)
                    .metal_view()
                    .vulkan()
                    .resizable()
                    .build()
                    .unwrap();
                todo!()
            })
            .expect("Main thread should be alive")
    }

    fn get_virgl_context(&self) -> Option<Arc<Mutex<webrogue_virgl::ContextContainer>>> {
        todo!()
    }
}
