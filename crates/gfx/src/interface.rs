wiggle::from_witx!({
    witx: ["witx/webrogue_gfx.witx"],
    wasmtime: false,
});

use std::sync::Arc;

use types::*;

pub struct GFXInterface {
    gfx: Arc<crate::system::GFXSystem>,
    window: Option<crate::window::Window>,
    gfxstream_thread: Option<webrogue_gfxstream::Thread>,
    dispatcher: Option<crate::DispatcherFunc>,
}

// gfx can be shared
// window can't TODO
// gfxstream_thread is not cloned/copied across threads
// TODO make wasi-threads not to force Send implementation
unsafe impl Send for GFXInterface {}

impl GFXInterface {
    pub fn new(gfx: Arc<crate::system::GFXSystem>) -> Self {
        let dispatcher = gfx.dispatcher;
        Self {
            gfx,
            window: None,
            gfxstream_thread: None,
            dispatcher,
        }
    }
}

impl Clone for GFXInterface {
    fn clone(&self) -> Self {
        Self {
            gfx: self.gfx.clone(),
            window: self.window.clone(),
            gfxstream_thread: None,
            dispatcher: self.dispatcher,
        }
    }
}

impl webrogue_gfx::WebrogueGfx for GFXInterface {
    fn make_window(&mut self, _mem: &mut wiggle::GuestMemory<'_>) {
        if self.window.is_some() {
            return;
        }

        self.window = Some(crate::dispatch::dispatch(self.dispatcher, || {
            self.gfx.make_window()
        }));
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

    fn gl_init(&mut self, _mem: &mut wiggle::GuestMemory<'_>) {
        if let Some(window) = &self.window {
            let ret = window.gl_init();
            self.gfxstream_thread = Some(webrogue_gfxstream::Thread::new(ret.0, ret.1))
        }
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
        let result = self.gfx.poll();
        let _ = mem.write(out_len, result);
    }

    fn poll_read(&mut self, mem: &mut wiggle::GuestMemory<'_>, buf: wiggle::GuestPtr<u8>) {
        let result = self.gfx.poll_read();
        if let Some(result) = result {
            let _ = mem.copy_from_slice(result, buf.as_array(result.len() as u32));
        }
    }

    fn get_gl_swap_interval(
        &mut self,
        mem: &mut wiggle::GuestMemory<'_>,
        out_interval: wiggle::GuestPtr<u32>,
    ) -> () {
        let result = self.gfx.get_gl_swap_interval();
        let _ = mem.write(out_interval, result);
    }
}
