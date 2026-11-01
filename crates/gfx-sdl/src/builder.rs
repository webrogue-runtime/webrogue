use std::{any::Any, sync::Arc};

use crate::{System, request_queue::RequestQueue};

pub struct SDLBuilder {}

impl SDLBuilder {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {})
        // SDLBuilder {}
        // sdl3::init()?;
    }

    pub fn run<Output>(
        self,
        body_fn: impl FnOnce(webrogue_gfx::System) -> Output + Send + 'static,
        vulkan_requirement: Option<bool>,
    ) -> anyhow::Result<Output>
    where
        Output: Send + 'static,
    {
        let sdl = sdl3::init()?;
        let event_subsystem = sdl.event()?;
        let video_subsystem = sdl.video()?;
        event_subsystem.register_custom_event::<MainThreadEvent>()?;
        let request_queue = Arc::new(RequestQueue::new(event_subsystem.event_sender()));
        let system = webrogue_gfx::System::new(Arc::new(System::new(request_queue.clone())));
        for event in sdl.event_pump()?.wait_iter() {
            if event.is_user_event() {
                let event = event.as_user_event_type::<MainThreadEvent>().unwrap();
                match event {
                    MainThreadEvent::Notify => todo!(),
                }
            }
            match event {
                _ => {}
            }
        }
        todo!()
    }
}

pub enum MainThreadEvent {
    Notify,
}
