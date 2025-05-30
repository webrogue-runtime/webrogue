wiggle::from_witx!({
    witx: ["witx/webrogue_gfx.witx"],
    wasmtime: false,
});

use std::sync::{Arc, Mutex};

use types::*;

pub trait ISystem<Window: IWindow> {
    fn make_window(&self) -> Window;
    fn poll(&self, events_buffer: &mut Vec<u8>);
    fn get_gl_swap_interval(&self) -> u32;
}
pub trait IWindow {
    fn present(&self);
    fn get_size(&self) -> (u32, u32);
    fn get_gl_size(&self) -> (u32, u32);
    fn gl_init(&mut self) -> (*const (), *const ());
}

pub struct Interface<System: ISystem<Window>, Window: IWindow> {
    system: Arc<System>,
    window: Option<Window>,
    gfxstream_thread: Option<webrogue_gfxstream::Thread>,
    event_buf: Arc<Mutex<Vec<u8>>>,
}

pub fn run<T, System: ISystem<Window> + 'static, Window: IWindow + 'static>(
    system: System,
    f: impl FnOnce(Interface<System, Window>) -> T,
) -> T {
    let interface = Interface::new(Arc::new(system));

    f(interface)
}

// gfx can be shared
// window can't TODO
// gfxstream_thread is not cloned/copied across threads
// TODO make wasi-threads not to force Send implementation
unsafe impl<System: ISystem<Window>, Window: IWindow> Send for Interface<System, Window> {}

impl<System: ISystem<Window>, Window: IWindow> Interface<System, Window> {
    pub fn new(system: Arc<System>) -> Self {
        // let dispatcher = gfx.dispatcher;
        Self {
            system,
            window: None,
            gfxstream_thread: None,
            event_buf: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<System: ISystem<Window>, Window: IWindow> Clone for Interface<System, Window> {
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            window: None,
            gfxstream_thread: None,
            event_buf: self.event_buf.clone(),
        }
    }
}

impl<System: ISystem<Window>, Window: IWindow> webrogue_gfx::WebrogueGfx
    for Interface<System, Window>
{
    fn make_window(&mut self, _mem: &mut wiggle::GuestMemory<'_>) {
        if self.window.is_some() {
            return;
        }

        self.window = Some(
            // crate::dispatch::dispatch(self.dispatcher, || {
            self.system.make_window(), // })
        );
    }

    fn present(&mut self, _mem: &mut wiggle::GuestMemory<'_>) {
        let Some(window) = self.window.as_mut() else {
            return;
        };
        window.present();
    }

    fn get_window_size(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        out_width: wiggle::GuestPtr<GfxSize>,
        out_height: wiggle::GuestPtr<GfxSize>,
    ) {
        let size = self
            .window
            .as_ref()
            .map(|window| window.get_size())
            .unwrap_or_default();
        let _ = mem.write(out_width, size.0);
        let _ = mem.write(out_height, size.1);
    }

    fn get_gl_size(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        out_width: wiggle::GuestPtr<GfxSize>,
        out_height: wiggle::GuestPtr<GfxSize>,
    ) {
        let size = self
            .window
            .as_ref()
            .map(|window| window.get_gl_size())
            .unwrap_or_default();
        let _ = mem.write(out_width, size.0);
        let _ = mem.write(out_height, size.1);
    }

    fn gl_init(&mut self, mem: &mut wiggle::GuestMemory<'_>, out_status: wiggle::GuestPtr<u8>) {
        let result = if let Some(window) = &mut self.window {
            let ret = window.gl_init();
            self.gfxstream_thread = Some(webrogue_gfxstream::Thread::new(ret.0, ret.1));
            true
        } else {
            false
        };
        let _ = mem.write(out_status, if result { 1 } else { 0 });
    }

    fn commit_buffer(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        buf: wiggle::GuestPtr<u8>,
        len: Size,
    ) {
        let Some(gfxstream_thread) = &self.gfxstream_thread else {
            return;
        };
        let Ok(b) = mem.as_cow(buf.as_array(len)) else {
            return;
        };
        gfxstream_thread.commit(&b);
    }

    fn ret_buffer_read(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        buf: wiggle::GuestPtr<u8>,
        len: Size,
    ) {
        let Some(gfxstream_thread) = &self.gfxstream_thread else {
            return;
        };
        let buffer = {
            let mut buffer = vec![0u8; len as usize];
            gfxstream_thread.read(&mut buffer);
            buffer
        };
        let _ = mem.copy_from_slice(&buffer, buf.as_array(len));
    }

    fn poll(&mut self, mem: &mut wiggle::GuestMemory<'_>, out_len: wiggle::GuestPtr<Size>) {
        let mut event_buf = self.event_buf.lock().unwrap();
        self.system.poll(&mut event_buf);
        let result = event_buf.len() as u32;
        let _ = mem.write(out_len, result);
    }

    fn poll_read(&mut self, mem: &mut wiggle::GuestMemory<'_>, buf: wiggle::GuestPtr<u8>) {
        let event_buf = self.event_buf.lock().unwrap();
        let _ = mem.copy_from_slice(&event_buf, buf.as_array(event_buf.len() as u32));
    }

    fn get_gl_swap_interval(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        out_interval: wiggle::GuestPtr<u32>,
    ) -> () {
        let result = self.system.get_gl_swap_interval();
        let _ = mem.write(out_interval, result);
    }
}
